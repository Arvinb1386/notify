package com.notify.companion.service

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.content.pm.ServiceInfo
import android.os.BatteryManager
import android.os.Build
import android.os.IBinder
import android.util.Log
import androidx.core.app.NotificationCompat
import com.notify.companion.network.CompanionClient
import com.notify.companion.network.CompanionMessage
import kotlinx.coroutines.*

class CompanionBackgroundService : Service() {

    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private var companionClient: CompanionClient? = null

    override fun onCreate() {
        super.onCreate()
        try {
            startForegroundNotification()
        } catch (e: Exception) {
            Log.e("CompanionService", "Foreground notification error", e)
        }

        try {
            companionClient = CompanionClient(applicationContext)
            companionClient?.startAutoConnection()
            NotificationCollectorService.companionClient = companionClient
        } catch (e: Exception) {
            Log.e("CompanionService", "Client start error", e)
        }

        // Periodic Battery & Wi-Fi Telemetry Streamer
        scope.launch {
            while (isActive) {
                try {
                    sendTelemetryUpdate()
                } catch (e: Exception) {
                    Log.e("CompanionService", "Telemetry error", e)
                }
                delay(5000)
            }
        }
    }

    private fun sendTelemetryUpdate() {
        val ifilter = IntentFilter(Intent.ACTION_BATTERY_CHANGED)
        val batteryStatusIntent: Intent? = registerReceiver(null, ifilter)

        val level: Int = batteryStatusIntent?.getIntExtra(BatteryManager.EXTRA_LEVEL, -1) ?: -1
        val scale: Int = batteryStatusIntent?.getIntExtra(BatteryManager.EXTRA_SCALE, -1) ?: -1
        val batteryPct: Int = if (level >= 0 && scale > 0) (level * 100 / scale) else 100

        val status: Int = batteryStatusIntent?.getIntExtra(BatteryManager.EXTRA_STATUS, -1) ?: -1
        val isCharging: Boolean = status == BatteryManager.BATTERY_STATUS_CHARGING || status == BatteryManager.BATTERY_STATUS_FULL
        val statusStr = if (isCharging) "charging" else "discharging"

        val tempRaw: Int = batteryStatusIntent?.getIntExtra(BatteryManager.EXTRA_TEMPERATURE, 250) ?: 250
        val tempCelsius = tempRaw / 10.0f

        val telemetry = CompanionMessage.Telemetry(
            batteryLevel = batteryPct,
            batteryStatus = statusStr,
            batteryTemp = tempCelsius,
            wifiSsid = "Connected Wi-Fi",
            wifiSignal = -50
        )

        companionClient?.sendMessage(telemetry.toJson())
    }

    private fun startForegroundNotification() {
        val channelId = "notify_companion_sync"
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                channelId,
                "Notify Sync Service",
                NotificationManager.IMPORTANCE_LOW
            )
            val manager = getSystemService(NotificationManager::class.java)
            manager?.createNotificationChannel(channel)
        }

        val notification: Notification = NotificationCompat.Builder(this, channelId)
            .setContentTitle("Notify Companion Active")
            .setContentText("Connected and syncing notifications with PC")
            .setSmallIcon(android.R.drawable.stat_notify_sync)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .build()

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            startForeground(101, notification, ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC)
        } else {
            startForeground(101, notification)
        }
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        super.onDestroy()
        scope.cancel()
    }
}
