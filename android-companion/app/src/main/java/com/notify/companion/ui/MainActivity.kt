package com.notify.companion.ui

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.PowerManager
import android.provider.Settings
import android.view.Gravity
import android.view.View
import android.widget.*
import androidx.appcompat.app.AppCompatActivity
import com.google.zxing.integration.android.IntentIntegrator
import com.notify.companion.network.CompanionClient
import com.notify.companion.service.CompanionBackgroundService

class MainActivity : AppCompatActivity() {

    private lateinit var statusText: TextView
    private lateinit var ipInput: EditText
    private lateinit var portInput: EditText
    private lateinit var connectBtn: Button
    private lateinit var qrScanBtn: Button
    private lateinit var autoScanBtn: Button
    private lateinit var permBtn: Button
    private lateinit var batteryBtn: Button
    private lateinit var companionClient: CompanionClient

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        companionClient = CompanionClient.getInstance(applicationContext)

        try {
            val scrollView = ScrollView(this)
            val layout = LinearLayout(this).apply {
                orientation = LinearLayout.VERTICAL
                setPadding(50, 60, 50, 60)
                setBackgroundColor(0xFF0F1117.toInt())
            }

            // App Brand Header with Icon
            val headerRow = LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                setPadding(0, 0, 0, 10)
            }

            val appLogo = ImageView(this).apply {
                setImageResource(android.R.drawable.ic_dialog_info)
                try {
                    val iconRes = resources.getIdentifier("ic_launcher", "drawable", packageName)
                    if (iconRes != 0) setImageResource(iconRes)
                } catch (_: Exception) {}
                layoutParams = LinearLayout.LayoutParams(90, 90).apply {
                    setMargins(0, 0, 24, 0)
                }
            }

            val titleBox = LinearLayout(this).apply {
                orientation = LinearLayout.VERTICAL
            }

            val title = TextView(this).apply {
                text = "Notify Companion"
                textSize = 22f
                setTextColor(0xFFFFFFFF.toInt())
                setTypeface(null, android.graphics.Typeface.BOLD)
            }

            val subtitle = TextView(this).apply {
                text = "Instant Wi-Fi Notification Mirror"
                textSize = 12f
                setTextColor(0xFF9E9E9E.toInt())
            }

            titleBox.addView(title)
            titleBox.addView(subtitle)
            headerRow.addView(appLogo)
            headerRow.addView(titleBox)

            statusText = TextView(this).apply {
                text = if (companionClient.isConnected) {
                    "Status: ● Connected to ${companionClient.currentServerAddress ?: "PC"}"
                } else {
                    "Status: ○ Auto-searching PC on Wi-Fi..."
                }
                textSize = 13f
                setTextColor(if (companionClient.isConnected) 0xFF10B981.toInt() else 0xFF818CF8.toInt())
                setPadding(0, 20, 0, 30)
            }

            // QR Code Scanner Button
            qrScanBtn = Button(this).apply {
                text = "📷 Scan PC QR Code (1-Click Pair)"
                setBackgroundColor(0xFF4F46E5.toInt())
                setTextColor(0xFFFFFFFF.toInt())
                setOnClickListener {
                    val integrator = IntentIntegrator(this@MainActivity)
                    integrator.setPrompt("Scan Notify QR Code on PC screen")
                    integrator.setOrientationLocked(false)
                    integrator.setBeepEnabled(true)
                    integrator.initiateScan()
                }
            }

            autoScanBtn = Button(this).apply {
                text = "🔍 Auto-Discover PC on Wi-Fi"
                setBackgroundColor(0xFF374151.toInt())
                setTextColor(0xFFFFFFFF.toInt())
                setOnClickListener {
                    statusText.text = "Status: Broadcasting UDP search on Wi-Fi..."
                    statusText.setTextColor(0xFFF59E0B.toInt())
                    companionClient.triggerQuickDiscovery()
                    Toast.makeText(this@MainActivity, "Searching for Notify PC...", Toast.LENGTH_SHORT).show()
                }
            }

            permBtn = Button(this).apply {
                setOnClickListener {
                    try {
                        startActivity(Intent("android.settings.ACTION_NOTIFICATION_LISTENER_SETTINGS"))
                    } catch (e: Exception) {
                        Toast.makeText(this@MainActivity, "Open Settings > Notification Access", Toast.LENGTH_SHORT).show()
                    }
                }
            }

            batteryBtn = Button(this).apply {
                setOnClickListener {
                    requestIgnoreBatteryOptimizations()
                }
            }

            val serverLabel = TextView(this).apply {
                text = "Manual PC Address:"
                textSize = 13f
                setTextColor(0xFFD1D5DB.toInt())
                setPadding(0, 30, 0, 10)
            }

            val prefs = getSharedPreferences("notify_companion_prefs", Context.MODE_PRIVATE)
            val savedIp = prefs.getString("server_ip", "") ?: ""
            val savedPort = prefs.getInt("server_port", 27890)

            ipInput = EditText(this).apply {
                hint = "PC IP (e.g. 192.168.1.4)"
                setText(savedIp)
                setHintTextColor(0xFF6B7280.toInt())
                setTextColor(0xFFFFFFFF.toInt())
                setBackgroundColor(0xFF181A20.toInt())
                setPadding(25, 25, 25, 25)
            }

            portInput = EditText(this).apply {
                hint = "Port (default: 27890)"
                setText(savedPort.toString())
                setHintTextColor(0xFF6B7280.toInt())
                setTextColor(0xFFFFFFFF.toInt())
                setBackgroundColor(0xFF181A20.toInt())
                setPadding(25, 25, 25, 25)
            }

            connectBtn = Button(this).apply {
                text = "Connect to PC"
                setBackgroundColor(0xFF10B981.toInt())
                setTextColor(0xFFFFFFFF.toInt())
                setOnClickListener {
                    val ipRaw = ipInput.text.toString().trim()
                    val port = portInput.text.toString().trim().toIntOrNull() ?: 27890
                    if (ipRaw.isNotEmpty()) {
                        // Supports a single IP or comma/space separated candidate list
                        val ipDisplay = ipRaw.split(',', ' ').firstOrNull { it.isNotBlank() } ?: ipRaw
                        statusText.text = "Status: Connecting to $ipDisplay:$port..."
                        statusText.setTextColor(0xFFF59E0B.toInt())
                        companionClient.connectWithQrData(ipRaw, port, "")
                        startBackgroundService()
                    } else {
                        Toast.makeText(this@MainActivity, "Please enter PC IP address", Toast.LENGTH_SHORT).show()
                    }
                }
            }

            layout.addView(headerRow)
            layout.addView(statusText)
            layout.addView(qrScanBtn)
            layout.addView(autoScanBtn)
            layout.addView(permBtn)
            layout.addView(batteryBtn)
            layout.addView(serverLabel)
            layout.addView(ipInput)
            layout.addView(portInput)
            layout.addView(connectBtn)

            // ---- Disconnect Button ----
            val disconnectBtn = Button(this).apply {
                text = "Disconnect from PC"
                setBackgroundColor(0xFFDC2626.toInt())
                setTextColor(0xFFFFFFFF.toInt())
                setOnClickListener {
                    if (companionClient.isConnected || !companionClient.userDisconnected) {
                        companionClient.disconnectUser()
                        statusText.text = "Status: ○ Disconnected. Tap Connect to pair again."
                        statusText.setTextColor(0xFF9E9E9E.toInt())
                        Toast.makeText(this@MainActivity, "Disconnected from PC", Toast.LENGTH_SHORT).show()

                        // Stop the background service so nothing reconnects silently
                        try {
                            stopService(Intent(this@MainActivity, CompanionBackgroundService::class.java))
                        } catch (_: Exception) {}
                    }
                }
            }
            layout.addView(disconnectBtn)

            scrollView.addView(layout)
            setContentView(scrollView)
        } catch (e: Exception) {
            e.printStackTrace()
        }

        setupClientListeners()
        startBackgroundService()
        updatePermissionButtons()
    }

    override fun onResume() {
        super.onResume()
        updateStatusView()
        updatePermissionButtons()
    }

    private fun isNotificationServiceEnabled(): Boolean {
        val flat = Settings.Secure.getString(contentResolver, "enabled_notification_listeners")
        return flat != null && flat.contains(packageName)
    }

    private fun isBatteryOptimizationIgnored(): Boolean {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            val pm = getSystemService(Context.POWER_SERVICE) as PowerManager
            return pm.isIgnoringBatteryOptimizations(packageName)
        }
        return true
    }

    private fun updatePermissionButtons() {
        if (isNotificationServiceEnabled()) {
            permBtn.text = "✓ 1. Notification Access: Granted"
            permBtn.setBackgroundColor(0xFF1E293B.toInt())
            permBtn.setTextColor(0xFF10B981.toInt())
        } else {
            permBtn.text = "1. Enable Notification Access (Required)"
            permBtn.setBackgroundColor(0xFF4F46E5.toInt())
            permBtn.setTextColor(0xFFFFFFFF.toInt())
        }

        if (isBatteryOptimizationIgnored()) {
            batteryBtn.text = "✓ 2. Battery Optimization: Disabled"
            batteryBtn.setBackgroundColor(0xFF1E293B.toInt())
            batteryBtn.setTextColor(0xFF10B981.toInt())
        } else {
            batteryBtn.text = "2. Disable Battery Optimization"
            batteryBtn.setBackgroundColor(0xFF374151.toInt())
            batteryBtn.setTextColor(0xFFFFFFFF.toInt())
        }
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        val result = IntentIntegrator.parseActivityResult(requestCode, resultCode, data)
        if (result != null) {
            if (result.contents != null) {
                val scanned = result.contents
                // Format: notify://pair?ip=192.168.1.4&ips=10.0.0.5&port=27890&secret=abc
                // "ip" is the primary LAN IP, "ips" holds alternate candidates
                // (comma-separated) which are auto-probed when a VPN is active.
                try {
                    val uri = Uri.parse(scanned)
                    val primaryIp = uri.getQueryParameter("ip")
                    val altIps = uri.getQueryParameter("ips")
                        ?.split(',')?.map { it.trim() }?.filter { it.isNotBlank() }
                        ?: emptyList()
                    val candidateIps = listOfNotNull(primaryIp) + altIps
                    val port = uri.getQueryParameter("port")?.toIntOrNull() ?: 27890
                    val secret = uri.getQueryParameter("secret") ?: ""

                    if (candidateIps.isNotEmpty()) {
                        ipInput.setText(candidateIps.joinToString(","))
                        portInput.setText(port.toString())
                        statusText.text = "Status: QR Scanned! Connecting to ${candidateIps.first()}:$port..."
                        statusText.setTextColor(0xFF10B981.toInt())
                        companionClient.connectWithQrData(candidateIps, port, secret)
                        startBackgroundService()
                        Toast.makeText(this, "QR Code Paired! Connecting...", Toast.LENGTH_SHORT).show()
                    }
                } catch (e: Exception) {
                    Toast.makeText(this, "Invalid QR Code format", Toast.LENGTH_SHORT).show()
                }
            }
        } else {
            super.onActivityResult(requestCode, resultCode, data)
        }
    }

    private fun setupClientListeners() {
        companionClient.onConnectionStateChanged = { isConnected, serverAddr ->
            if (isConnected) {
                statusText.text = "Status: ● Connected to ${serverAddr ?: "PC"}"
                statusText.setTextColor(0xFF10B981.toInt())
                Toast.makeText(this@MainActivity, "Connected to PC!", Toast.LENGTH_SHORT).show()
            } else {
                statusText.text = "Status: ○ Reconnecting / Searching PC..."
                statusText.setTextColor(0xFFF59E0B.toInt())
            }
        }

        companionClient.onDiscoveryFound = { ips, port, name ->
            ipInput.setText(ips.joinToString(","))
            portInput.setText(port.toString())
            statusText.text = "Status: ● Found $name at ${ips.first()}:$port! Connecting..."
            statusText.setTextColor(0xFF10B981.toInt())
        }
    }

    private fun updateStatusView() {
        if (companionClient.isConnected) {
            statusText.text = "Status: ● Connected to ${companionClient.currentServerAddress ?: "PC"}"
            statusText.setTextColor(0xFF10B981.toInt())
        } else if (companionClient.userDisconnected) {
            statusText.text = "Status: ○ Disconnected manually"
            statusText.setTextColor(0xFF9E9E9E.toInt())
        }
    }

    private fun startBackgroundService() {
        try {
            val serviceIntent = Intent(this, CompanionBackgroundService::class.java)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                startForegroundService(serviceIntent)
            } else {
                startService(serviceIntent)
            }
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }

    private fun requestIgnoreBatteryOptimizations() {
        try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                val pm = getSystemService(Context.POWER_SERVICE) as PowerManager
                if (!pm.isIgnoringBatteryOptimizations(packageName)) {
                    val intent = Intent(Settings.ACTION_REQUEST_IGNORE_BATTERY_OPTIMIZATIONS).apply {
                        data = Uri.parse("package:$packageName")
                    }
                    startActivity(intent)
                } else {
                    Toast.makeText(this, "Battery optimization is already disabled!", Toast.LENGTH_SHORT).show()
                }
            }
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }
}
