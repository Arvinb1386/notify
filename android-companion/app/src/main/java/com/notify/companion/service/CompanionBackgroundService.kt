package com.notify.companion.service

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import android.util.Log
import androidx.core.app.NotificationCompat
import com.notify.companion.network.CompanionClient

class CompanionBackgroundService : Service() {

    private lateinit var companionClient: CompanionClient

    override fun onCreate() {
        super.onCreate()
        try {
            startForegroundNotification()
        } catch (e: Exception) {
            Log.e("CompanionService", "Foreground notification error", e)
        }

        try {
            companionClient = CompanionClient.getInstance(applicationContext)
            companionClient.startAutoConnection()
            // Note: battery/storage telemetry is owned by CompanionClient (starts
            // with every WebSocket connection) — this service only keeps the
            // process alive in the foreground.
        } catch (e: Exception) {
            Log.e("CompanionService", "Client start error", e)
        }
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
}
