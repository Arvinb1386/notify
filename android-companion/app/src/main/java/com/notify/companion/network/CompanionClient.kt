package com.notify.companion.network

import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.os.BatteryManager
import android.os.Build
import android.os.Environment
import android.os.Handler
import android.os.Looper
import android.os.StatFs
import android.util.Log
import kotlinx.coroutines.*
import okhttp3.*
import org.json.JSONObject
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress
import java.net.InetSocketAddress
import java.net.Socket
import java.util.concurrent.TimeUnit

class CompanionClient(private val context: Context) {

    companion object {
        @Volatile
        var sharedClient: CompanionClient? = null

        fun getInstance(context: Context): CompanionClient {
            return sharedClient ?: synchronized(this) {
                sharedClient ?: CompanionClient(context.applicationContext).also {
                    it.registerUnderlyingNetworkMonitor()
                    sharedClient = it
                }
            }
        }
    }

    private val client = OkHttpClient.Builder()
        .readTimeout(0, TimeUnit.MILLISECONDS)
        .pingInterval(8, TimeUnit.SECONDS)
        .retryOnConnectionFailure(true)
        .build()

    // ---------------------------------------------------------------------------
    // Underlying (non-VPN) network binding.
    //
    // When the phone's VPN is active, Android routes ALL traffic — including UDP
    // broadcast and plain LAN traffic — into the tunnel by default. That made the
    // companion unable to reach the PC on real Wi-Fi ("reconnecting" forever).
    // We grab the underlying Wi-Fi/Ethernet Network via ConnectivityManager and
    // explicitly bind every discovery socket / TCP probe / WebSocket to it,
    // bypassing the tunnel entirely.
    // ---------------------------------------------------------------------------
    @Volatile private var underlyingNetwork: Network? = null

    @Volatile private var webSocketClient: OkHttpClient = client

    private fun registerUnderlyingNetworkMonitor() {
        try {
            val cm = context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
            val request = NetworkRequest.Builder()
                .addTransportType(NetworkCapabilities.TRANSPORT_WIFI)
                .addTransportType(NetworkCapabilities.TRANSPORT_ETHERNET)
                .build()

            cm.registerNetworkCallback(request, object : ConnectivityManager.NetworkCallback() {
                override fun onAvailable(network: Network) {
                    underlyingNetwork = network
                    rebuildWebSocketClient()
                    Log.i("CompanionClient", "Underlying Wi-Fi/Ethernet network available: $network (VPN bypass active)")
                }

                override fun onLost(network: Network) {
                    if (underlyingNetwork == network) {
                        underlyingNetwork = null
                        rebuildWebSocketClient()
                        Log.w("CompanionClient", "Underlying network lost — falling back to default routing")
                    }
                }
            })

            // Pick up any already-active Wi-Fi immediately (callback only fires on changes)
            for (network in cm.allNetworks) {
                val caps = cm.getNetworkCapabilities(network)
                if (caps != null &&
                    caps.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) &&
                    !caps.hasTransport(NetworkCapabilities.TRANSPORT_VPN)
                ) {
                    underlyingNetwork = network
                    rebuildWebSocketClient()
                    break
                }
            }
        } catch (e: Exception) {
            Log.e("CompanionClient", "Network monitor registration failed", e)
        }
    }

    /** OkHttp client that routes through the underlying Wi-Fi when one exists */
    private fun rebuildWebSocketClient() {
        webSocketClient = underlyingNetwork?.let { network ->
            client.newBuilder()
                .socketFactory(network.socketFactory)
                .dns(object : Dns {
                    override fun lookup(hostname: String): List<InetAddress> =
                        network.getAllByName(hostname).toList()
                })
                .build()
        } ?: client
    }

    private var webSocket: WebSocket? = null
    var isConnected = false
        private set

    var currentServerAddress: String? = null
        private set

    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private val mainHandler = Handler(Looper.getMainLooper())

    /** Set when the user pressed Disconnect — blocks auto-reconnect until they connect manually again */
    @Volatile
    var userDisconnected = false
        private set

    /** Guards against spawning multiple auto-reconnect loops (service restarts) */
    private var autoConnectStarted = false

    /** Guards against spawning multiple telemetry loops */
    private var telemetryLoopStarted = false

    var onQuickReplyReceived: ((key: String, text: String) -> Unit)? = null
    var onConnectionStateChanged: ((Boolean, String?) -> Unit)? = null
    var onDiscoveryFound: ((List<String>, Int, String) -> Unit)? = null
    var onDiscoverySearching: (() -> Unit)? = null

    private val sharedPrefs = context.getSharedPreferences("notify_companion_prefs", Context.MODE_PRIVATE)

    /** A PC server found via UDP broadcast, with every advertised LAN IP candidate */
    data class DiscoveredServer(val ips: List<String>, val port: Int, val name: String)

    fun startAutoConnection() {
        if (autoConnectStarted) return
        autoConnectStarted = true

        scope.launch {
            while (isActive) {
                if (!isConnected && !userDisconnected) {
                    val savedIps = loadSavedCandidates()

                    // 1. Try UDP broadcast auto-discovery first
                    Log.d("CompanionClient", "Searching local Wi-Fi via UDP Discovery...")
                    val discovered = discoverServerViaUdp()
                    if (discovered != null && discovered.ips.isNotEmpty()) {
                        Log.i("CompanionClient", "Auto-discovered Notify PC: ${discovered.name} at ${discovered.ips}:${discovered.port}")
                        mainHandler.post {
                            onDiscoveryFound?.invoke(discovered.ips, discovered.port, discovered.name)
                        }
                        saveCandidate(discovered.ips.first(), discovered.port)
                        connectToServers(discovered.ips, discovered.port)
                    } else if (savedIps.isNotEmpty()) {
                        // 2. Fallback to saved candidates (rotates through all known IPs)
                        Log.d("CompanionClient", "Connecting to saved candidates $savedIps")
                        connectToServers(savedIps, loadSavedPort())
                    }
                }
                delay(4000)
            }
        }
    }

    fun triggerQuickDiscovery() {
        // A manual search is explicit intent to connect
        userDisconnected = false
        mainHandler.post { onDiscoverySearching?.invoke() }
        scope.launch {
            val discovered = discoverServerViaUdp()
            if (discovered != null && discovered.ips.isNotEmpty()) {
                mainHandler.post {
                    onDiscoveryFound?.invoke(discovered.ips, discovered.port, discovered.name)
                }
                connectToServers(discovered.ips, discovered.port)
            }
        }
    }

    private suspend fun discoverServerViaUdp(): DiscoveredServer? = withContext(Dispatchers.IO) {
        var socket: DatagramSocket? = null
        try {
            socket = DatagramSocket()
            underlyingNetwork?.bindSocket(socket)
            socket.broadcast = true
            socket.reuseAddress = true
            socket.soTimeout = 1500

            val sendData = "NOTIFY_DISCOVER".toByteArray()
            val broadcastAddr = InetAddress.getByName("255.255.255.255")
            val sendPacket = DatagramPacket(sendData, sendData.size, broadcastAddr, 27891)
            socket.send(sendPacket)

            // Collect ALL responses within the timeout window: the PC answers with one
            // packet per LAN IP candidate, some of which may be unreachable VPN IPs.
            val deadline = System.currentTimeMillis() + 1500
            val foundIps = LinkedHashSet<String>()
            var port = 27890
            var name = "Notify PC"

            while (System.currentTimeMillis() < deadline) {
                val receiveBuf = ByteArray(1024)
                val receivePacket = DatagramPacket(receiveBuf, receiveBuf.size)
                try {
                    socket.receive(receivePacket)
                } catch (_: Exception) {
                    break // socket timeout
                }

                val response = String(receivePacket.data, 0, receivePacket.length).trim()
                if (response.startsWith("NOTIFY_SERVER")) {
                    val parts = response.split("|")
                    if (parts.size >= 3) {
                        parts[1].split(",").map { it.trim() }.filter { it.isNotBlank() }.forEach {
                            foundIps.add(it)
                        }
                        port = parts[2].toIntOrNull() ?: 27890
                        if (parts.size >= 4) name = parts[3]
                    }
                }
            }

            if (foundIps.isEmpty()) null else DiscoveredServer(foundIps.toList(), port, name)
        } catch (e: Exception) {
            Log.d("CompanionClient", "UDP discovery timeout or error: ${e.message}")
            null
        } finally {
            try {
                socket?.close()
            } catch (_: Exception) {}
        }
    }

    fun connectWithQrData(ip: String, port: Int, secret: String) {
        // Accept single IP or comma/space separated candidate list (e.g. from QR "ips" param)
        val candidates = ip.split(',', ' ')
            .map { it.trim() }
            .filter { it.isNotBlank() && it.contains('.') }
        connectWithCandidates(candidates, port, secret)
    }

    fun connectWithQrData(ips: List<String>, port: Int, secret: String) =
        connectWithCandidates(ips, port, secret)

    private fun connectWithCandidates(candidates: List<String>, port: Int, secret: String) {
        val primary = candidates.firstOrNull() ?: return

        // Manual connection always clears a previous user-disconnect request
        userDisconnected = false

        saveCandidates(candidates, port)
        sharedPrefs.edit().putString("pairing_secret", secret).apply()

        scope.launch {
            connectToServers(candidates, port)
        }
    }

    /**
     * User-initiated disconnect from the phone side. Closes the WebSocket and
     * stops auto-reconnect loops until the user connects manually again.
     */
    fun disconnectUser() {
        Log.i("CompanionClient", "User requested disconnect")
        userDisconnected = true
        isConnected = false
        currentServerAddress = null
        try {
            webSocket?.close(1000, "user_disconnect")
        } catch (_: Exception) {}
        try {
            webSocket?.cancel()
        } catch (_: Exception) {}
        webSocket = null
        mainHandler.post {
            onConnectionStateChanged?.invoke(false, null)
        }
    }

    // ---- Candidate persistence helpers ----

    private fun loadSavedCandidates(): List<String> {
        val raw = sharedPrefs.getString("server_ips", null)
            ?: sharedPrefs.getString("server_ip", null)
            ?: return emptyList()
        return raw.split(',').map { it.trim() }.filter { it.contains('.') }
    }

    private fun loadSavedPort(): Int = sharedPrefs.getInt("server_port", 27890)

    private fun saveCandidate(ip: String, port: Int) {
        // Merge with previously known candidates so we never "forget" IPs that
        // only appeared once (e.g. PC briefly advertising its VPN tunnel IP).
        val merged = LinkedHashSet(loadSavedCandidates())
        merged.add(ip)
        saveCandidates(merged.toList(), port)
    }

    private fun saveCandidates(ips: List<String>, port: Int) {
        sharedPrefs.edit()
            .putString("server_ips", ips.joinToString(","))
            .putString("server_ip", ips.firstOrNull() ?: "")
            .putInt("server_port", port)
            .apply()
    }

    /**
     * Probes every candidate LAN IP IN PARALLEL and opens the WebSocket to the
     * best reachable one (original advertisement order wins ties). Under an
     * active VPN the PC may advertise tunnel IPs (10.x / 100.x etc.) that the
     * phone can't reach — those fail the TCP probe within ~800ms and are
     * skipped instead of breaking pairing.
     */
    private suspend fun connectToServers(candidates: List<String>, port: Int) {
        val distinct = candidates.distinct()
        val reachable = distinct.map { candidate ->
            scope.async(Dispatchers.IO) {
                if (isHostReachable(candidate, port)) candidate else null
            }
        }.awaitAll().filterNotNull()

        if (reachable.isEmpty()) {
            Log.w("CompanionClient", "No reachable PC among candidates $distinct:$port")
            mainHandler.post {
                onConnectionStateChanged?.invoke(false, null)
            }
            return
        }

        // Preserve the PC's advertised priority order
        val winner = distinct.first { it in reachable }
        Log.i("CompanionClient", "PC reachable at $winner:$port (${reachable.size}/${distinct.size} candidates responded), opening WebSocket")
        saveCandidate(winner, port)
        connectToWebSocket(winner, port)
    }

    /** Fast TCP reachability probe (800ms timeout), bound to real Wi-Fi when a VPN is active */
    private fun isHostReachable(ip: String, port: Int): Boolean {
        return try {
            Socket().use { socket ->
                underlyingNetwork?.bindSocket(socket)
                socket.connect(InetSocketAddress(ip, port), 800)
                true
            }
        } catch (_: Exception) {
            false
        }
    }

    private fun connectToWebSocket(ip: String, port: Int) {
        try {
            webSocket?.cancel()
            val request = Request.Builder()
                .url("ws://$ip:$port")
                .build()

            currentServerAddress = "$ip:$port"

            // webSocketClient routes through the underlying Wi-Fi when a VPN is active
            webSocket = webSocketClient.newWebSocket(request, object : WebSocketListener() {
                override fun onOpen(ws: WebSocket, response: Response) {
                    Log.i("CompanionClient", "WebSocket Connected to $ip:$port")
                    isConnected = true
                    mainHandler.post {
                        onConnectionStateChanged?.invoke(true, "$ip:$port")
                    }

                    // Send Handshake
                    val deviceId = getOrCreateDeviceId()
                    val pairingSecret = sharedPrefs.getString("pairing_secret", "") ?: ""
                    val handshake = CompanionMessage.Handshake(
                        deviceId = deviceId,
                        deviceName = "${Build.MANUFACTURER} ${Build.MODEL}",
                        manufacturer = Build.MANUFACTURER,
                        model = Build.MODEL,
                        androidVersion = Build.VERSION.RELEASE,
                        pairingToken = pairingSecret
                    )
                    sendMessage(handshake.toJson())

                    // Battery/storage telemetry is owned by the client itself so it
                    // survives service restarts / user disconnects and always resumes
                    // as soon as the WebSocket is open.
                    startTelemetryLoop()
                }

                override fun onMessage(ws: WebSocket, text: String) {
                    try {
                        val json = JSONObject(text)
                        val type = json.optString("type")
                        val payload = json.optJSONObject("payload")

                        if (type == "quick_reply" && payload != null) {
                            val key = payload.getString("key")
                            val replyText = payload.getString("reply_text")
                            mainHandler.post {
                                onQuickReplyReceived?.invoke(key, replyText)
                            }
                        }
                    } catch (e: Exception) {
                        Log.e("CompanionClient", "Error parsing server message", e)
                    }
                }

                override fun onClosed(ws: WebSocket, code: Int, reason: String) {
                    Log.w("CompanionClient", "WebSocket Closed: $reason")
                    isConnected = false
                    mainHandler.post {
                        onConnectionStateChanged?.invoke(false, null)
                    }
                }

                override fun onFailure(ws: WebSocket, t: Throwable, response: Response?) {
                    Log.e("CompanionClient", "WebSocket Failure: ${t.message}")
                    isConnected = false
                    mainHandler.post {
                        onConnectionStateChanged?.invoke(false, null)
                    }
                }
            })
        } catch (e: Exception) {
            Log.e("CompanionClient", "Failed to connect WebSocket", e)
            isConnected = false
            mainHandler.post {
                onConnectionStateChanged?.invoke(false, null)
            }
        }
    }

    fun sendMessage(jsonString: String) {
        if (isConnected) {
            webSocket?.send(jsonString)
        }
    }

    // ---- Battery & Storage Telemetry ----

    private fun startTelemetryLoop() {
        if (telemetryLoopStarted) return
        telemetryLoopStarted = true

        scope.launch {
            while (isActive) {
                if (isConnected) {
                    try {
                        sendTelemetryUpdate()
                    } catch (e: Exception) {
                        Log.e("CompanionClient", "Telemetry error", e)
                    }
                }
                delay(4000)
            }
        }
    }

    private fun sendTelemetryUpdate() {
        val ifilter = IntentFilter(Intent.ACTION_BATTERY_CHANGED)
        val batteryStatusIntent: Intent? = context.registerReceiver(null, ifilter)

        val level: Int = batteryStatusIntent?.getIntExtra(BatteryManager.EXTRA_LEVEL, -1) ?: -1
        val scale: Int = batteryStatusIntent?.getIntExtra(BatteryManager.EXTRA_SCALE, -1) ?: -1
        val batteryPct: Int = if (level >= 0 && scale > 0) (level * 100 / scale) else 100

        val status: Int = batteryStatusIntent?.getIntExtra(BatteryManager.EXTRA_STATUS, -1) ?: -1
        val isCharging: Boolean =
            status == BatteryManager.BATTERY_STATUS_CHARGING || status == BatteryManager.BATTERY_STATUS_FULL
        val statusStr = if (isCharging) "charging" else "discharging"

        val tempRaw: Int = batteryStatusIntent?.getIntExtra(BatteryManager.EXTRA_TEMPERATURE, 250) ?: 250
        val tempCelsius = tempRaw / 10.0f

        // Internal storage stats (fixes the empty Storage widget in companion mode)
        var freeGb = 0.0
        var totalGb = 0.0
        try {
            val stat = StatFs(Environment.getDataDirectory().path)
            totalGb = stat.totalBytes / (1024.0 * 1024.0 * 1024.0)
            freeGb = stat.availableBytes / (1024.0 * 1024.0 * 1024.0)
        } catch (_: Exception) {}

        val telemetry = CompanionMessage.Telemetry(
            batteryLevel = batteryPct,
            batteryStatus = statusStr,
            batteryTemp = tempCelsius,
            wifiSsid = null, // Wi-Fi SSID requires location permission; the PC shows a fallback
            wifiSignal = null,
            storageFreeGb = freeGb,
            storageTotalGb = totalGb
        )

        Log.d("CompanionClient", "Sending telemetry: battery=$batteryPct% ($statusStr), storage ${"%.1f".format(freeGb)}/${"%.1f".format(totalGb)}GB")
        sendMessage(telemetry.toJson())
    }

    /** Re-enables the auto-connect loop after a manual disconnect (e.g. app relaunch) */
    fun clearUserDisconnect() {
        userDisconnected = false
    }

    private fun getOrCreateDeviceId(): String {
        var id = sharedPrefs.getString("device_uuid", null)
        if (id == null) {
            id = java.util.UUID.randomUUID().toString()
            sharedPrefs.edit().putString("device_uuid", id).apply()
        }
        return id
    }
}
