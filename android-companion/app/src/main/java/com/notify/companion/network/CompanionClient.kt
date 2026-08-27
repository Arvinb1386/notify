package com.notify.companion.network

import android.content.Context
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.util.Log
import kotlinx.coroutines.*
import okhttp3.*
import org.json.JSONObject
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress
import java.util.concurrent.TimeUnit

class CompanionClient(private val context: Context) {

    companion object {
        @Volatile
        var sharedClient: CompanionClient? = null

        fun getInstance(context: Context): CompanionClient {
            return sharedClient ?: synchronized(this) {
                sharedClient ?: CompanionClient(context.applicationContext).also {
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

    private var webSocket: WebSocket? = null
    var isConnected = false
        private set

    var currentServerAddress: String? = null
        private set

    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private val mainHandler = Handler(Looper.getMainLooper())

    var onQuickReplyReceived: ((key: String, text: String) -> Unit)? = null
    var onConnectionStateChanged: ((Boolean, String?) -> Unit)? = null
    var onDiscoveryFound: ((List<String>, Int, String) -> Unit)? = null

    private val sharedPrefs = context.getSharedPreferences("notify_companion_prefs", Context.MODE_PRIVATE)

    /** A PC server found via UDP broadcast, with every advertised LAN IP candidate */
    data class DiscoveredServer(val ips: List<String>, val port: Int, val name: String)

    fun startAutoConnection() {
        scope.launch {
            while (isActive) {
                if (!isConnected) {
                    val savedIp = sharedPrefs.getString("server_ip", null)
                    val savedPort = sharedPrefs.getInt("server_port", 27890)

                    // 1. Try UDP broadcast auto-discovery first
                    Log.d("CompanionClient", "Searching local Wi-Fi via UDP Discovery...")
                    val discovered = discoverServerViaUdp()
                    if (discovered != null && discovered.ips.isNotEmpty()) {
                        Log.i("CompanionClient", "Auto-discovered Notify PC: ${discovered.name} at ${discovered.ips}:${discovered.port}")
                        mainHandler.post {
                            onDiscoveryFound?.invoke(discovered.ips, discovered.port, discovered.name)
                        }
                        sharedPrefs.edit()
                            .putString("server_ip", discovered.ips.first())
                            .putInt("server_port", discovered.port)
                            .apply()
                        connectToServers(discovered.ips, discovered.port)
                    } else if (!savedIp.isNullOrBlank()) {
                        // 2. Fallback to saved IP
                        Log.d("CompanionClient", "Connecting to saved IP $savedIp:$savedPort")
                        connectToServers(listOf(savedIp), savedPort)
                    }
                }
                delay(4000)
            }
        }
    }

    fun triggerQuickDiscovery() {
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
        sharedPrefs.edit()
            .putString("server_ip", primary)
            .putInt("server_port", port)
            .putString("pairing_secret", secret)
            .apply()

        scope.launch {
            connectToServers(candidates, port)
        }
    }

    /**
     * Probes every candidate LAN IP and opens the WebSocket to the first one that
     * actually responds. Under an active VPN the PC may advertise tunnel IPs
     * (10.x / 100.x etc.) that the phone can't reach on LAN — those fail the TCP
     * probe within ~800ms each and are skipped instead of breaking pairing.
     */
    private suspend fun connectToServers(candidates: List<String>, port: Int) {
        for (candidate in candidates.distinct()) {
            if (!isHostReachable(candidate, port)) {
                Log.d("CompanionClient", "Skipping unreachable candidate $candidate:$port (likely a VPN/tunnel IP)")
                continue
            }
            Log.i("CompanionClient", "PC reachable at $candidate:$port, opening WebSocket")
            connectToWebSocket(candidate, port)
            return
        }

        Log.w("CompanionClient", "No reachable PC among candidates $candidates:$port")
        mainHandler.post {
            onConnectionStateChanged?.invoke(false, null)
        }
    }

    /** Fast TCP reachability probe (800ms timeout) */
    private fun isHostReachable(ip: String, port: Int): Boolean {
        return try {
            java.net.Socket().use { socket ->
                socket.connect(java.net.InetSocketAddress(ip, port), 800)
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

            webSocket = client.newWebSocket(request, object : WebSocketListener() {
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

    private fun getOrCreateDeviceId(): String {
        var id = sharedPrefs.getString("device_uuid", null)
        if (id == null) {
            id = java.util.UUID.randomUUID().toString()
            sharedPrefs.edit().putString("device_uuid", id).apply()
        }
        return id
    }
}
