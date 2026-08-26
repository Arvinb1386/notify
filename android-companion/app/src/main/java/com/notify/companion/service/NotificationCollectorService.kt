package com.notify.companion.service

import android.app.Notification
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

        val appName = try {
            val pm = applicationContext.packageManager
            val appInfo = pm.getApplicationInfo(pkg, 0)
            pm.getApplicationLabel(appInfo).toString()
        } catch (e: Exception) {
            pkg
        }

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
