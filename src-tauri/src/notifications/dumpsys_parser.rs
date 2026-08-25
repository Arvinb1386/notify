use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use super::otp_detector::OtpDetector;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NotificationStatus {
    Posted,
    Updated,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotificationItem {
    pub id: String,
    pub package_name: String,
    pub app_name: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub subtext: Option<String>,
    pub channel_id: Option<String>,
    pub post_time: i64,
    pub is_otp: bool,
    pub otp_code: Option<String>,
    pub status: NotificationStatus,
    pub fingerprint: String,
}

lazy_static! {
    // Matches NotificationRecord headers in dumpsys output across Android 10 - 15
    // Example: NotificationRecord(0x76b...: pkg=com.whatsapp user=UserHandle{0} id=12 tag=null: ...
    static ref RECORD_HEADER: Regex = Regex::new(
        r"NotificationRecord\([^:]+:\s*pkg=([a-zA-Z0-9_.]+)\s+user=UserHandle\{(\d+)\}\s+id=(\d+)\s+tag=([^:\s]+)"
    ).unwrap();

    static ref TITLE_REGEX: Regex = Regex::new(r"android\.title=(?:String\s*\((.*?)\)|([^\n\r]+))").unwrap();
    static ref TEXT_REGEX: Regex = Regex::new(r"android\.text=(?:String\s*\((.*?)\)|([^\n\r]+))").unwrap();
    static ref BIG_TEXT_REGEX: Regex = Regex::new(r"android\.bigText=(?:String\s*\((.*?)\)|([^\n\r]+))").unwrap();
    static ref SUBTEXT_REGEX: Regex = Regex::new(r"android\.subText=(?:String\s*\((.*?)\)|([^\n\r]+))").unwrap();
    static ref POST_TIME_REGEX: Regex = Regex::new(r"postTime=(\d+)").unwrap();
    static ref CHANNEL_REGEX: Regex = Regex::new(r"NotificationChannel\{mId='([^']+)'").unwrap();
}

pub struct DumpsysParser;

impl DumpsysParser {
    /// Parses raw output from `adb shell dumpsys notification --noredact`
    pub fn parse_snapshot(dumpsys_output: &str) -> Vec<NotificationItem> {
        let mut items = Vec::new();

        // Split output by NotificationRecord boundaries
        let record_chunks: Vec<&str> = dumpsys_output.split("NotificationRecord(").collect();

        for chunk in record_chunks.into_iter().skip(1) {
            let full_chunk = format!("NotificationRecord({}", chunk);

            if let Some(caps) = RECORD_HEADER.captures(&full_chunk) {
                let pkg = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                let user_id = caps.get(2).map(|m| m.as_str()).unwrap_or("0");
                let notif_id = caps.get(3).map(|m| m.as_str()).unwrap_or("0");
                let tag = caps.get(4).map(|m| m.as_str()).unwrap_or("null");

                // Filter out system persistent notifications (like low battery warning or usb debugging)
                if pkg == "android" && full_chunk.contains("FLAG_ONGOING_EVENT") && notif_id == "26" {
                    continue; // Skip persistent ADB debugging notification itself
                }

                let title = TITLE_REGEX.captures(&full_chunk).and_then(|c| {
                    c.get(1).or_else(|| c.get(2)).map(|m| m.as_str().trim().to_string())
                }).filter(|s| s != "null" && !s.is_empty());

                let big_text = BIG_TEXT_REGEX.captures(&full_chunk).and_then(|c| {
                    c.get(1).or_else(|| c.get(2)).map(|m| m.as_str().trim().to_string())
                }).filter(|s| s != "null" && !s.is_empty());

                let text = TEXT_REGEX.captures(&full_chunk).and_then(|c| {
                    c.get(1).or_else(|| c.get(2)).map(|m| m.as_str().trim().to_string())
                }).filter(|s| s != "null" && !s.is_empty());

                let body = big_text.or(text);

                let subtext = SUBTEXT_REGEX.captures(&full_chunk).and_then(|c| {
                    c.get(1).or_else(|| c.get(2)).map(|m| m.as_str().trim().to_string())
                }).filter(|s| s != "null" && !s.is_empty());

                let post_time: i64 = POST_TIME_REGEX
                    .captures(&full_chunk)
                    .and_then(|c| c.get(1))
                    .and_then(|m| m.as_str().parse::<i64>().ok())
                    .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

                let channel_id = CHANNEL_REGEX
                    .captures(&full_chunk)
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string());

                let id = format!("{}_{}_{}_{}", pkg, user_id, notif_id, tag);
                let app_name = Self::resolve_app_name(&pkg);

                // Run OTP detection
                let otp_res = OtpDetector::detect(title.as_deref(), body.as_deref());
                let (is_otp, otp_code) = match otp_res {
                    Some(otp) => (true, Some(otp.code)),
                    None => (false, None),
                };

                // Compute unique fingerprint
                let mut hasher = Sha256::new();
                hasher.update(format!("{}:{}:{}:{}", pkg, title.as_deref().unwrap_or(""), body.as_deref().unwrap_or(""), post_time));
                let fingerprint = format!("{:x}", hasher.finalize());

                items.push(NotificationItem {
                    id,
                    package_name: pkg,
                    app_name,
                    title,
                    body,
                    subtext,
                    channel_id,
                    post_time,
                    is_otp,
                    otp_code,
                    status: NotificationStatus::Posted,
                    fingerprint,
                });
            }
        }

        items
    }

    /// Maps common package names to readable display names
    pub fn resolve_app_name(pkg: &str) -> Option<String> {
        let mut map = HashMap::new();
        map.insert("com.whatsapp", "WhatsApp");
        map.insert("org.telegram.messenger", "Telegram");
        map.insert("org.telegram.plus", "Telegram Plus");
        map.insert("org.thunderdog.challegram", "Telegram X");
        map.insert("com.google.android.apps.messaging", "Messages");
        map.insert("com.samsung.android.messaging", "Samsung Messages");
        map.insert("com.google.android.gm", "Gmail");
        map.insert("com.instagram.android", "Instagram");
        map.insert("com.twitter.android", "X (Twitter)");
        map.insert("com.discord", "Discord");
        map.insert("com.slack", "Slack");
        map.insert("com.spotify.music", "Spotify");
        map.insert("ir.divar", "Divar");
        map.insert("ir.snapp.passenger", "Snapp");
        map.insert("ir.tapsi.cab", "Tapsi");
        map.insert("ir.mci.ecareapp", "Hamrah Man");
        map.insert("ir.irancell.myirancell", "MyIrancell");
        map.insert("ir.tejaratbank.mobilebank", "Tejarat Mobile");
        map.insert("ir.mellatbank.mobilebank", "Mellat Mobile");
        map.insert("com.bale.messenger", "Bale");
        map.insert("ir.eitaa.messenger", "Eitaa");
        map.insert("ir.resaneh.rubika", "Rubika");

        map.get(pkg).map(|&s| s.to_string())
    }
}
