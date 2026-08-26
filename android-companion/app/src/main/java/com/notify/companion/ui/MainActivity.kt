package com.notify.companion.ui

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.PowerManager
import android.provider.Settings
import android.widget.*
import androidx.appcompat.app.AppCompatActivity
import com.notify.companion.network.CompanionClient
import com.notify.companion.service.CompanionBackgroundService

class MainActivity : AppCompatActivity() {

    private lateinit var statusText: TextView
    private lateinit var ipInput: EditText
    private lateinit var portInput: EditText
    private lateinit var connectBtn: Button
    private lateinit var permBtn: Button
    private lateinit var batteryBtn: Button

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        try {
            // Setup simple, clean UI layout programmatically
            val scrollView = ScrollView(this)
            val layout = LinearLayout(this).apply {
                orientation = LinearLayout.VERTICAL
                setPadding(60, 60, 60, 60)
                setBackgroundColor(0xFF0F1117.toInt())
            }

            val title = TextView(this).apply {
                text = "Notify Companion"
                textSize = 22f
                setTextColor(0xFFFFFFFF.toInt())
                setTypeface(null, android.graphics.Typeface.BOLD)
            }

            val subtitle = TextView(this).apply {
                text = "Connect to PC and mirror notifications instantly without ADB"
                textSize = 12f
                setTextColor(0xFF9E9E9E.toInt())
                setPadding(0, 10, 0, 30)
            }

            statusText = TextView(this).apply {
                text = "Status: Ready to connect"
                textSize = 13f
                setTextColor(0xFF818CF8.toInt())
                setPadding(0, 0, 0, 30)
            }

            permBtn = Button(this).apply {
                text = "1. Enable Notification Access"
                setBackgroundColor(0xFF4F46E5.toInt())
                setTextColor(0xFFFFFFFF.toInt())
                setOnClickListener {
                    try {
                        startActivity(Intent("android.settings.ACTION_NOTIFICATION_LISTENER_SETTINGS"))
                    } catch (e: Exception) {
                        Toast.makeText(this@MainActivity, "Open Settings > Notification Access", Toast.LENGTH_SHORT).show()
                    }
                }
            }

            batteryBtn = Button(this).apply {
                text = "2. Disable Battery Optimization"
                setBackgroundColor(0xFF374151.toInt())
                setTextColor(0xFFFFFFFF.toInt())
                setOnClickListener {
                    requestIgnoreBatteryOptimizations()
                }
            }

            val serverLabel = TextView(this).apply {
                text = "PC Server Address:"
                textSize = 13f
                setTextColor(0xFFD1D5DB.toInt())
                setPadding(0, 30, 0, 10)
            }

            ipInput = EditText(this).apply {
                hint = "PC IP (e.g. 192.168.1.5)"
                setHintTextColor(0xFF6B7280.toInt())
                setTextColor(0xFFFFFFFF.toInt())
                setBackgroundColor(0xFF181A20.toInt())
                setPadding(25, 25, 25, 25)
            }

            portInput = EditText(this).apply {
                hint = "Port (default: 27890)"
                setText("27890")
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
                    val ip = ipInput.text.toString().trim()
                    val port = portInput.text.toString().trim().toIntOrNull() ?: 27890
                    if (ip.isNotEmpty()) {
                        val client = CompanionClient(applicationContext)
                        client.connectWithQrData(ip, port, "")
                        startBackgroundService()
                        statusText.text = "Status: Connecting to $ip:$port..."
                        statusText.setTextColor(0xFF10B981.toInt())
                        Toast.makeText(this@MainActivity, "Connecting to PC...", Toast.LENGTH_SHORT).show()
                    } else {
                        Toast.makeText(this@MainActivity, "Please enter PC IP address", Toast.LENGTH_SHORT).show()
                    }
                }
            }

            layout.addView(title)
            layout.addView(subtitle)
            layout.addView(statusText)
            layout.addView(permBtn)
            layout.addView(batteryBtn)
            layout.addView(serverLabel)
            layout.addView(ipInput)
            layout.addView(portInput)
            layout.addView(connectBtn)

            scrollView.addView(layout)
            setContentView(scrollView)
        } catch (e: Exception) {
            e.printStackTrace()
        }

        try {
            startBackgroundService()
        } catch (e: Exception) {
            e.printStackTrace()
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
