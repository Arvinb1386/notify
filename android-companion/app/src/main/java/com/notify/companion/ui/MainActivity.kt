package com.notify.companion.ui

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.PowerManager
import android.provider.Settings
import android.graphics.Color
import android.graphics.Typeface
import android.graphics.drawable.GradientDrawable
import android.view.Gravity
import android.view.View
import android.view.ViewGroup
import android.widget.*
import androidx.appcompat.app.AppCompatActivity
import androidx.appcompat.app.AlertDialog
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
    private lateinit var devicesContainer: LinearLayout
    private lateinit var companionClient: CompanionClient

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        companionClient = CompanionClient.getInstance(applicationContext)
        // Re-opening the app counts as intent to connect — clear any previous
        // in-app "Disconnect" so the auto-connect loop resumes.
        companionClient.clearUserDisconnect()

        try {
            val scrollView = ScrollView(this)
            val layout = LinearLayout(this).apply {
                orientation = LinearLayout.VERTICAL
                setPadding(dp(22), dp(28), dp(22), dp(32))
                setBackgroundColor(Color.rgb(15, 17, 23))
            }

            // App Brand Header with Icon
            val headerRow = LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                setPadding(0, 0, 0, dp(16))
            }

            val appLogo = ImageView(this).apply {
                setImageResource(android.R.drawable.ic_dialog_info)
                try {
                    val iconRes = resources.getIdentifier("ic_launcher", "drawable", packageName)
                    if (iconRes != 0) setImageResource(iconRes)
                } catch (_: Exception) {}
                layoutParams = LinearLayout.LayoutParams(dp(58), dp(58)).apply {
                    setMargins(0, 0, dp(16), 0)
                }
            }

            val titleBox = LinearLayout(this).apply {
                orientation = LinearLayout.VERTICAL
                layoutParams = LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f)
            }

            val title = TextView(this).apply {
                text = "Notify Companion"
                textSize = 24f
                setTextColor(Color.rgb(245, 247, 255))
                setTypeface(null, Typeface.BOLD)
            }

            val subtitle = TextView(this).apply {
                text = "Instant Wi-Fi Notification Mirror"
                textSize = 13f
                setTextColor(Color.rgb(164, 169, 188))
            }

            titleBox.addView(title)
            titleBox.addView(subtitle)
            val settingsBtn = TextView(this).apply {
                text = "⚙"
                textSize = 28f
                gravity = Gravity.CENTER
                setTextColor(Color.rgb(220, 224, 240))
                background = rounded(Color.rgb(28, 31, 41), Color.rgb(62, 67, 84), 1)
                layoutParams = LinearLayout.LayoutParams(dp(52), dp(52))
                setOnClickListener { showSettingsDialog() }
            }

            headerRow.addView(appLogo)
            headerRow.addView(titleBox)
            headerRow.addView(settingsBtn)

            statusText = TextView(this).apply {
                text = if (companionClient.isConnected) {
                    "Status: ● Connected to ${companionClient.currentServerAddress ?: "PC"}"
                } else {
                    "Status: ○ Auto-searching PC on Wi-Fi..."
                }
                textSize = 14f
                setTypeface(null, Typeface.BOLD)
                setTextColor(if (companionClient.isConnected) Color.rgb(20, 130, 85) else Color.rgb(79, 70, 229))
                setPadding(0, dp(8), 0, dp(20))
            }

            // QR Code Scanner Button
            qrScanBtn = Button(this).apply {
                text = "📷 Scan PC QR Code (1-Click Pair)"
                stylePrimary(this)
                setTextColor(Color.rgb(255, 255, 255))
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
                styleSecondary(this)
                setTextColor(Color.rgb(225, 228, 240))
                setOnClickListener {
                    statusText.text = "Status: Broadcasting UDP search on Wi-Fi..."
                    statusText.setTextColor(Color.rgb(255, 190, 80))
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

            val devicesLabel = TextView(this).apply {
                text = "Available devices"
                textSize = 18f
                setTypeface(null, Typeface.BOLD)
                setTextColor(Color.rgb(245, 247, 255))
                setPadding(0, dp(28), 0, dp(8))
            }

            devicesContainer = LinearLayout(this).apply {
                orientation = LinearLayout.VERTICAL
            }

            val serverLabel = TextView(this).apply {
                text = "Manual connection"

                textSize = 14f
                setTypeface(null, Typeface.BOLD)
                setTextColor(Color.rgb(225, 228, 240))
                setPadding(0, dp(26), 0, dp(10))
            }

            val prefs = getSharedPreferences("notify_companion_prefs", Context.MODE_PRIVATE)
            val savedIp = prefs.getString("server_ip", "") ?: ""
            val savedPort = prefs.getInt("server_port", 27890)

            ipInput = EditText(this).apply {
                hint = "PC IP (e.g. 192.168.1.4)"
                setText(savedIp)
                setHintTextColor(Color.rgb(145, 148, 160))
                setTextColor(Color.rgb(235, 238, 248))
                background = rounded(Color.rgb(28, 31, 41), Color.rgb(62, 67, 84), 1)
                setPadding(dp(16), dp(14), dp(16), dp(14))
            }

            portInput = EditText(this).apply {
                hint = "Port (default: 27890)"
                setText(savedPort.toString())
                setHintTextColor(Color.rgb(145, 148, 160))
                setTextColor(Color.rgb(235, 238, 248))
                background = rounded(Color.rgb(28, 31, 41), Color.rgb(62, 67, 84), 1)
                setPadding(dp(16), dp(14), dp(16), dp(14))
            }

            connectBtn = Button(this).apply {
                text = "Connect to PC"
                styleSuccess(this)
                setTextColor(Color.rgb(255, 255, 255))
                setOnClickListener {
                    val ipRaw = ipInput.text.toString().trim()
                    val port = portInput.text.toString().trim().toIntOrNull() ?: 27890
                    if (ipRaw.isNotEmpty()) {
                        // Supports a single IP or comma/space separated candidate list
                        val ipDisplay = ipRaw.split(',', ' ').firstOrNull { it.isNotBlank() } ?: ipRaw
                        statusText.text = "Status: Connecting to $ipDisplay:$port..."
                        statusText.setTextColor(Color.rgb(255, 190, 80))
                        companionClient.connectWithQrData(ipRaw, port, "")
                        startBackgroundService()
                    } else {
                        Toast.makeText(this@MainActivity, "Please enter PC IP address", Toast.LENGTH_SHORT).show()
                    }
                }
            }

            layout.addView(headerRow)
            layout.addView(statusText)
            layout.addView(qrScanBtn, buttonParams())
            layout.addView(autoScanBtn, buttonParams())
            layout.addView(devicesLabel)
            layout.addView(devicesContainer, LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT
            ).apply { bottomMargin = dp(10) })
            layout.addView(serverLabel)
            layout.addView(ipInput)
            layout.addView(portInput)
            layout.addView(connectBtn, buttonParams())

            // ---- Disconnect Button ----
            val disconnectBtn = Button(this).apply {
                text = "Disconnect from PC"
                background = rounded(Color.rgb(55, 27, 32), Color.rgb(125, 65, 75), 1)
                setTextColor(Color.rgb(255, 125, 125))
                setOnClickListener {
                    if (companionClient.isConnected || !companionClient.userDisconnected) {
                        companionClient.disconnectUser()
                        statusText.text = "Status: ○ Disconnected. Tap Connect to pair again."
                        statusText.setTextColor(Color.rgb(164, 169, 188))
                        Toast.makeText(this@MainActivity, "Disconnected from PC", Toast.LENGTH_SHORT).show()

                        // Stop the background service so nothing reconnects silently
                        try {
                            stopService(Intent(this@MainActivity, CompanionBackgroundService::class.java))
                        } catch (_: Exception) {}
                    }
                }
            }
            layout.addView(disconnectBtn, buttonParams())

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

    private fun dp(value: Int): Int = (value * resources.displayMetrics.density).toInt()

    private fun showSettingsDialog() {
        val content = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(dp(22), dp(4), dp(22), dp(8))
            setBackgroundColor(Color.rgb(22, 25, 34))
        }

        val hint = TextView(this).apply {
            text = "Permissions keep Notify Companion connected and able to mirror notifications."
            textSize = 13f
            setTextColor(Color.rgb(164, 169, 188))
            setPadding(0, 0, 0, dp(12))
        }
        content.addView(hint)
        content.addView(permBtn, LinearLayout.LayoutParams(
            ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT
        ).apply { setMargins(0, dp(5), 0, dp(5)) })
        content.addView(batteryBtn, LinearLayout.LayoutParams(
            ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT
        ).apply { setMargins(0, dp(5), 0, dp(5)) })

        val dialog = AlertDialog.Builder(this)
            .setTitle("Settings")
            .setView(content)
            .setNegativeButton("Close", null)
            .create()
        dialog.show()
        dialog.setOnShowListener {
            dialog.window?.setBackgroundDrawable(rounded(Color.rgb(22, 25, 34)))
            dialog.getButton(AlertDialog.BUTTON_NEGATIVE)?.setTextColor(Color.rgb(170, 160, 255))
        }
    }

    private fun buttonParams(): LinearLayout.LayoutParams = LinearLayout.LayoutParams(
        ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT
    ).apply { setMargins(0, dp(6), 0, dp(6)) }

    private fun rounded(fill: Int, stroke: Int? = null, strokeWidth: Int = 0): GradientDrawable =
        GradientDrawable().apply {
            shape = GradientDrawable.RECTANGLE
            cornerRadius = dp(16).toFloat()
            setColor(fill)
            if (stroke != null) setStroke(dp(strokeWidth), stroke)
        }

    private fun stylePrimary(button: Button) {
        button.background = rounded(Color.rgb(79, 70, 229))
        button.minHeight = dp(52)
        button.setPadding(dp(16), 0, dp(16), 0)
        button.textSize = 14f
        button.isAllCaps = false
    }

    private fun styleSecondary(button: Button) {
        button.background = rounded(Color.rgb(28, 31, 41), Color.rgb(62, 67, 84), 1)
        button.minHeight = dp(52)
        button.setPadding(dp(16), 0, dp(16), 0)
        button.textSize = 14f
        button.isAllCaps = false
    }

    private fun styleSuccess(button: Button) {
        button.background = rounded(Color.rgb(24, 148, 96))
        button.minHeight = dp(52)
        button.setPadding(dp(16), 0, dp(16), 0)
        button.textSize = 14f
        button.isAllCaps = false
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
            styleSuccess(permBtn)
            permBtn.setTextColor(Color.rgb(255, 255, 255))
        } else {
            permBtn.text = "1. Notification Access (only needed for mirroring notifications)"
            stylePrimary(permBtn)
            permBtn.setTextColor(Color.rgb(255, 255, 255))
        }

        if (isBatteryOptimizationIgnored()) {
            batteryBtn.text = "✓ 2. Battery Optimization: Disabled"
            styleSuccess(batteryBtn)
            batteryBtn.setTextColor(Color.rgb(255, 255, 255))
        } else {
            batteryBtn.text = "2. Disable Battery Optimization"
            styleSecondary(batteryBtn)
            batteryBtn.setTextColor(Color.rgb(225, 228, 240))
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
                        statusText.setTextColor(Color.rgb(70, 220, 145))
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
                statusText.setTextColor(Color.rgb(70, 220, 145))
                Toast.makeText(this@MainActivity, "Connected to PC!", Toast.LENGTH_SHORT).show()
            } else {
                statusText.text = "Status: ○ Reconnecting / Searching PC..."
                statusText.setTextColor(Color.rgb(255, 190, 80))
            }
        }

        companionClient.onDiscoverySearching = {
            devicesContainer.removeAllViews()
            val searching = TextView(this@MainActivity).apply {
                text = "Searching for PCs on this Wi-Fi…"
                textSize = 14f
                setTextColor(Color.rgb(164, 169, 188))
                setPadding(dp(16), dp(16), dp(16), dp(16))
                background = rounded(Color.rgb(28, 31, 41), Color.rgb(62, 67, 84), 1)
            }
            devicesContainer.addView(searching)
        }

        companionClient.onDiscoveryFound = { ips, port, name ->
            devicesContainer.removeAllViews()
            ips.distinct().forEachIndexed { index, ip ->
                val device = TextView(this@MainActivity).apply {
                    text = "🖥  $name\n     $ip:$port"
                    textSize = 15f
                    setTypeface(null, Typeface.BOLD)
                    setTextColor(Color.rgb(235, 238, 248))
                    gravity = Gravity.CENTER_VERTICAL
                    setPadding(dp(18), dp(14), dp(18), dp(14))
                    background = rounded(Color.rgb(28, 31, 41), Color.rgb(62, 67, 84), 1)
                    setOnClickListener {
                        ipInput.setText(ip)
                        portInput.setText(port.toString())
                        statusText.text = "Status: Connecting to $ip:$port…"
                        statusText.setTextColor(Color.rgb(190, 120, 20))
                        companionClient.connectWithQrData(ip, port, "")
                        startBackgroundService()
                    }
                }
                devicesContainer.addView(device, LinearLayout.LayoutParams(
                    ViewGroup.LayoutParams.MATCH_PARENT, dp(76)
                ).apply { setMargins(0, if (index == 0) 0 else dp(10), 0, 0) })
            }
            statusText.text = "Status: Select a device to connect"
            statusText.setTextColor(Color.rgb(79, 70, 229))
        }
    }

    private fun updateStatusView() {
        if (companionClient.isConnected) {
            statusText.text = "Status: ● Connected to ${companionClient.currentServerAddress ?: "PC"}"
            statusText.setTextColor(Color.rgb(70, 220, 145))
        } else if (companionClient.userDisconnected) {
            statusText.text = "Status: ○ Disconnected manually"
            statusText.setTextColor(Color.rgb(164, 169, 188))
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
