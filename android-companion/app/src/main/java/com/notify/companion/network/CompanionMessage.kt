package com.notify.companion.network

import org.json.JSONObject

sealed class CompanionMessage {
    data class Handshake(
        val deviceId: String,
        val deviceName: String,
        val manufacturer: String,
        val model: String,
        val androidVersion: String,
        val pairingToken: String
    ) : CompanionMessage() {
        fun toJson(): String {
            val root = JSONObject()
            root.put("type", "handshake")
            val payload = JSONObject()
            payload.put("device_id", deviceId)
            payload.put("device_name", deviceName)
            payload.put("manufacturer", manufacturer)
            payload.put("model", model)
            payload.put("android_version", androidVersion)
            payload.put("pairing_token", pairingToken)
            root.put("payload", payload)
            return root.toString()
        }
    }

    data class NotificationPosted(
        val key: String,
        val packageName: String,
        val appName: String,
        val title: String?,
        val body: String?,
        val subtext: String?,
        val postTime: Long,
        val iconBase64: String?,
        val canReply: Boolean
    ) : CompanionMessage() {
        fun toJson(): String {
            val root = JSONObject()
            root.put("type", "notification_posted")
            val payload = JSONObject()
            payload.put("key", key)
            payload.put("package_name", packageName)
            payload.put("app_name", appName)
            payload.put("title", title ?: JSONObject.NULL)
            payload.put("body", body ?: JSONObject.NULL)
            payload.put("subtext", subtext ?: JSONObject.NULL)
            payload.put("post_time", postTime)
            payload.put("icon_base64", iconBase64 ?: JSONObject.NULL)
            payload.put("can_reply", canReply)
            root.put("payload", payload)
            return root.toString()
        }
    }

    data class NotificationRemoved(
        val key: String,
        val packageName: String
    ) : CompanionMessage() {
        fun toJson(): String {
            val root = JSONObject()
            root.put("type", "notification_removed")
            val payload = JSONObject()
            payload.put("key", key)
            payload.put("package_name", packageName)
            root.put("payload", payload)
            return root.toString()
        }
    }

    data class Telemetry(
        val batteryLevel: Int,
        val batteryStatus: String,
        val batteryTemp: Float,
        val wifiSsid: String?,
        val wifiSignal: Int?,
        val storageFreeGb: Double = 0.0,
        val storageTotalGb: Double = 0.0
    ) : CompanionMessage() {
        fun toJson(): String {
            val root = JSONObject()
            root.put("type", "telemetry")
            val payload = JSONObject()
            payload.put("battery_level", batteryLevel)
            payload.put("battery_status", batteryStatus)
            payload.put("battery_temp", batteryTemp)
            payload.put("wifi_ssid", wifiSsid ?: JSONObject.NULL)
            payload.put("wifi_signal", wifiSignal ?: JSONObject.NULL)
            payload.put("storage_free_gb", storageFreeGb)
            payload.put("storage_total_gb", storageTotalGb)
            root.put("payload", payload)
            return root.toString()
        }
    }
}
