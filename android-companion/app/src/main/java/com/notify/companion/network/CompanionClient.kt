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
    var onDiscoveryFound: ((String, Int, String) -> Unit)? = null

    private val sharedPrefs = context.getSharedPreferences("notify_companion_prefs", Context.MODE_PRIVATE)

    fun startAutoConnection() {
        scope.launch {
            while (isActive) {
                if (!isConnected) {
                    val savedIp = sharedPrefs.getString("server_ip", null)
                    val savedPort = sharedPrefs.getInt("server_port", 27890)

                    // 1. Try UDP broadcast auto-discovery first
                    Log.d("CompanionClient", "Searching local Wi-Fi via UDP Discovery...")
                    val discovered = discoverServerViaUdp()
                    if (discovered != null) {
                        val (ip, port, name) = discovered
                        Log.i("CompanionClient", "Auto-discovered Notify PC: $name at $ip:$port")
                        mainHandler.post {
                            onDiscoveryFound?.invoke(ip, port, name)
                        }
                        sharedPrefs.edit().putString("server_ip", ip).putInt("server_port", port).apply()
                        connectToWebSocket(ip, port)
                    } else if (!savedIp.isNullOrBlank()) {
                        // 2. Fallback to saved IP
                        Log.d("CompanionClient", "Connecting to saved IP $savedIp:$savedPort")
                        connectToWebSocket(savedIp, savedPort)
                    }
                }
                delay(4000)
            }
        }
    }

    fun triggerQuickDiscovery() {
        scope.launch {
            val discovered = discoverServerViaUdp()
            if (discovered != null) {
                val (ip, port, name) = discovered
                mainHandler.post {
                    onDiscoveryFound?.invoke(ip, port, name)
                }
                connectWithQrData(ip, port, "")
            }
        }
    }

    private suspend fun discoverServerViaUdp(): Triple<String, Int, String>? = withContext(Dispatchers.IO) {
        var socket: DatagramSocket? = null
        try {
            socket = DatagramSocket()
            socket.broadcast = true
            socket.soTimeout = 1500

            val sendData = "NOTIFY_DISCOVER".toByteArray()
            val broadcastAddr = InetAddress.getByName("255.255.255.255")
            val sendPacket = DatagramPacket(sendData, sendData.size, broadcastAddr, 27891)
            socket.send(sendPacket)

            val receiveBuf = ByteArray(1024)
            val receivePacket = DatagramPacket(receiveBuf, receiveBuf.size)
            socket.receive(receivePacket)

            val response = String(receivePacket.data, 0, receivePacket.length).trim()

            if (response.startsWith("NOTIFY_SERVER")) {
                val parts = response.split("|")
                if (parts.size >= 3) {
                    val ip = parts[1]
                    val port = parts[2].toIntOrNull() ?: 27890
                    val name = if (parts.size >= 4) parts[3] else "Notify PC"
                    return@withContext Triple(ip, port, name)
                }
            }
        } catch (e: Exception) {
            Log.d("CompanionClient", "UDP discovery timeout or error: ${e.message}")
        } finally {
            try {
                socket?.close()
            } catch (_: Exception) {}
        }
        return@withContext null
    }

    fun connectWithQrData(ip: String, port: Int, secret: String) {
        sharedPrefs.edit()
            .putString("server_ip", ip)
            .putInt("server_port", port)
            .putString("pairing_secret", secret)
            .apply()

        scope.launch {
            connectToWebSocket(ip, port)
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
