package com.notify.companion.network

import android.content.Context
import android.os.Build
import android.util.Log
import kotlinx.coroutines.*
import okhttp3.*
import org.json.JSONObject
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress
import java.util.concurrent.TimeUnit

class CompanionClient(private val context: Context) {

    private val client = OkHttpClient.Builder()
        .readTimeout(0, TimeUnit.MILLISECONDS)
        .pingInterval(10, TimeUnit.SECONDS)
        .build()

    private var webSocket: WebSocket? = null
    private var isConnected = false
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    var onQuickReplyReceived: ((key: String, text: String) -> Unit)? = null
    var onConnectionStateChanged: ((Boolean) -> Unit)? = null

    private val sharedPrefs = context.getSharedPreferences("notify_companion_prefs", Context.MODE_PRIVATE)

    fun startAutoConnection() {
        scope.launch {
            while (isActive) {
                if (!isConnected) {
                    val savedIp = sharedPrefs.getString("server_ip", null)
                    val savedPort = sharedPrefs.getInt("server_port", 27890)

                    if (savedIp != null) {
                        Log.d("CompanionClient", "Attempting connection to $savedIp:$savedPort")
                        connectToWebSocket(savedIp, savedPort)
                    } else {
                        // Scan network via UDP broadcast beacon
                        Log.d("CompanionClient", "No saved IP. Broadcasting UDP discovery...")
                        val discovered = discoverServerViaUdp()
                        if (discovered != null) {
                            val (ip, port) = discovered
                            sharedPrefs.edit().putString("server_ip", ip).putInt("server_port", port).apply()
                            connectToWebSocket(ip, port)
                        }
                    }
                }
                delay(4000)
            }
        }
    }

    private suspend fun discoverServerViaUdp(): Pair<String, Int>? = withContext(Dispatchers.IO) {
        try {
            val socket = DatagramSocket()
            socket.broadcast = true
            socket.soTimeout = 2000

            val sendData = "NOTIFY_DISCOVER".toByteArray()
            val broadcastAddr = InetAddress.getByName("255.255.255.255")
            val sendPacket = DatagramPacket(sendData, sendData.size, broadcastAddr, 27891)
            socket.send(sendPacket)

            val receiveBuf = ByteArray(1024)
            val receivePacket = DatagramPacket(receiveBuf, receiveBuf.size)
            socket.receive(receivePacket)

            val response = String(receivePacket.data, 0, receivePacket.length)
            socket.close()

            if (response.startsWith("NOTIFY_SERVER")) {
                val parts = response.split("|")
                if (parts.size >= 3) {
                    val ip = parts[1]
                    val port = parts[2].toIntOrNull() ?: 27890
                    return@withContext Pair(ip, port)
                }
            }
        } catch (e: Exception) {
            Log.d("CompanionClient", "UDP discovery timeout or error: ${e.message}")
        }
        return@withContext null
    }

    fun connectWithQrData(ip: String, port: Int, secret: String) {
        sharedPrefs.edit()
            .putString("server_ip", ip)
            .putInt("server_port", port)
            .putString("pairing_secret", secret)
            .apply()

        connectToWebSocket(ip, port)
    }

    private fun connectToWebSocket(ip: String, port: Int) {
        try {
            val request = Request.Builder()
                .url("ws://$ip:$port")
                .build()

            webSocket = client.newWebSocket(request, object : WebSocketListener() {
                override fun onOpen(ws: WebSocket, response: Response) {
                    Log.i("CompanionClient", "WebSocket Connected to $ip:$port")
                    isConnected = true
                    onConnectionStateChanged?.invoke(true)

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
                            onQuickReplyReceived?.invoke(key, replyText)
                        }
                    } catch (e: Exception) {
                        Log.e("CompanionClient", "Error parsing server message", e)
                    }
                }

                override fun onClosed(ws: WebSocket, code: Int, reason: String) {
                    Log.w("CompanionClient", "WebSocket Closed: $reason")
                    isConnected = false
                    onConnectionStateChanged?.invoke(false)
                }

                override fun onFailure(ws: WebSocket, t: Throwable, response: Response?) {
                    Log.e("CompanionClient", "WebSocket Failure: ${t.message}")
                    isConnected = false
                    onConnectionStateChanged?.invoke(false)
                }
            })
        } catch (e: Exception) {
            Log.e("CompanionClient", "Failed to connect WebSocket", e)
            isConnected = false
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
