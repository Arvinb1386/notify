package com.notify.companion.service

import android.app.Notification
import android.content.Context
import android.content.pm.LauncherApps
import android.service.notification.NotificationListenerService
import android.service.notification.StatusBarNotification
import android.util.Log
import com.notify.companion.network.CompanionClient
import com.notify.companion.network.CompanionMessage

class NotificationCollectorService : NotificationListenerService() {

    private lateinit var companionClient: CompanionClient

    override fun onCreate() {
        super.onCreate()
        companionClient = CompanionClient.getInstance(applicationContext)
        companionClient.startAutoConnection()
        Log.i("NotificationCollector", "Notification Listener Service Started")
    }

    /**
     * Resolves a human-readable app label. The plain PackageManager lookup only
     * covers the current user — apps installed in a work profile / parallel
     * space / secure folder throw NameNotFoundException, which previously made
     * us fall back to the raw package name (com.whatsapp instead of WhatsApp).
     */
    private fun resolveAppName(pkg: String, sbn: StatusBarNotification): String {
        val pm = applicationContext.packageManager

        // 1. Standard lookup (current user)
        try {
            val appInfo = pm.getApplicationInfo(pkg, 0)
            val label = pm.getApplicationLabel(appInfo).toString()
            if (label.isNotBlank()) return label
        } catch (_: Exception) {
        }

        // 2. The notification's own user handle (work profile / parallel space)
        try {
            val launcherApps = getSystemService(Context.LAUNCHER_APPS_SERVICE) as LauncherApps
            val label = launcherApps.getActivityList(pkg, sbn.user).firstOrNull()?.label?.toString()
            if (!label.isNullOrBlank()) return label
        } catch (_: Exception) {
        }

        Log.w("NotificationCollector", "Could not resolve app label for $pkg — falling back to package name")
        return pkg
    }

    override fun onNotificationPosted(sbn: StatusBarNotification?) {
        if (sbn == null) return

        val pkg = sbn.packageName ?: return
        val notif = sbn.notification ?: return
        val extras = notif.extras ?: return

        // Skip system ongoing notifications
        if ((notif.flags and Notification.FLAG_ONGOING_EVENT) != 0 && (pkg == "android" || pkg == "com.android.systemui")) {
            return
        }

        val title = extras.getCharSequence(Notification.EXTRA_TITLE)?.toString()
        val text = extras.getCharSequence(Notification.EXTRA_TEXT)?.toString()
        val bigText = extras.getCharSequence(Notification.EXTRA_BIG_TEXT)?.toString()
        val subtext = extras.getCharSequence(Notification.EXTRA_SUB_TEXT)?.toString()

        val body = bigText ?: text
        if (title.isNullOrBlank() && body.isNullOrBlank()) return

        val appName = resolveAppName(pkg, sbn)

        val key = sbn.key ?: "${pkg}_${sbn.id}"
        val postTime = sbn.postTime

        val msg = CompanionMessage.NotificationPosted(
            key = key,
            packageName = pkg,
            appName = appName,
            title = title,
            body = body,
            subtext = subtext,
            postTime = postTime,
            iconBase64 = null,
            canReply = true
        )

        Log.d("NotificationCollector", "Sending notification from $appName: $title")
        companionClient.sendMessage(msg.toJson())
    }

    override fun onNotificationRemoved(sbn: StatusBarNotification?) {
        if (sbn == null) return
        val pkg = sbn.packageName ?: return
        val key = sbn.key ?: "${pkg}_${sbn.id}"

        val msg = CompanionMessage.NotificationRemoved(key, pkg)
        companionClient.sendMessage(msg.toJson())
    }
}
