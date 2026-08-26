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
    // Universal NotificationRecord Header regex across Android 10, 11, 12, 13, 14, 15, OneUI, MIUI
    static ref RECORD_HEADER: Regex = Regex::new(
        r"NotificationRecord\([^:]+:\s*pkg=([a-zA-Z0-9_.]+)\s+user=UserHandle\{(-?\d+)\}\s+id=(-?\d+)\s+tag=([^:\s]+)"
    ).unwrap();

    static ref KEY_FALLBACK: Regex = Regex::new(
        r"key=(-?\d+)\|([a-zA-Z0-9_.]+)\|(-?\d+)\|([^|\s]+)"
    ).unwrap();

    static ref TITLE_REGEX: Regex = Regex::new(r"android\.title=(?:String\s*\((.*?)\)|([^\n\r]+))").unwrap();
    static ref TEXT_REGEX: Regex = Regex::new(r"android\.text=(?:String\s*\((.*?)\)|([^\n\r]+))").unwrap();
    static ref BIG_TEXT_REGEX: Regex = Regex::new(r"android\.bigText=(?:String\s*\((.*?)\)|([^\n\r]+))").unwrap();
    static ref SUBTEXT_REGEX: Regex = Regex::new(r"android\.subText=(?:String\s*\((.*?)\)|([^\n\r]+))").unwrap();
    static ref TICKER_REGEX: Regex = Regex::new(r"tickerText=([^\n\r]+)").unwrap();
    static ref WHEN_REGEX: Regex = Regex::new(r"when=(\d{10,14})").unwrap();
    static ref POST_TIME_REGEX: Regex = Regex::new(r"postTime=(\d+)").unwrap();
    static ref CHANNEL_REGEX: Regex = Regex::new(r"(?:NotificationChannel\{mId='|channel=)([^'\s,]+)").unwrap();
}

pub struct DumpsysParser;

impl DumpsysParser {
    /// Parses raw output from `adb shell dumpsys notification --noredact`
    pub fn parse_snapshot(dumpsys_output: &str) -> Vec<NotificationItem> {
        let mut items = Vec::new();
        let mut seen_content_keys = std::collections::HashSet::new();

        // Split output by NotificationRecord boundaries
        let record_chunks: Vec<&str> = dumpsys_output.split("NotificationRecord(").collect();

        for chunk in record_chunks.into_iter().skip(1) {
            let full_chunk = format!("NotificationRecord({}", chunk);

            let (pkg, user_id, notif_id, tag) = if let Some(caps) = RECORD_HEADER.captures(&full_chunk) {
                (
                    caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
                    caps.get(2).map(|m| m.as_str()).unwrap_or("0").to_string(),
                    caps.get(3).map(|m| m.as_str()).unwrap_or("0").to_string(),
                    caps.get(4).map(|m| m.as_str()).unwrap_or("null").to_string(),
                )
            } else if let Some(caps) = KEY_FALLBACK.captures(&full_chunk) {
                (
                    caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default(),
                    caps.get(1).map(|m| m.as_str()).unwrap_or("0").to_string(),
                    caps.get(3).map(|m| m.as_str()).unwrap_or("0").to_string(),
                    caps.get(4).map(|m| m.as_str()).unwrap_or("null").to_string(),
                )
            } else {
                continue;
            };

            // Skip persistent low-priority system debug notification
            if (pkg == "android" || pkg == "com.android.systemui")
                && (full_chunk.contains("Wireless debugging") || full_chunk.contains("USB debugging") || full_chunk.contains("ZEN_ONGOING"))
            {
                continue;
            }

            // Skip Android group summary container notifications (flags=0x200 or FLAG_GROUP_SUMMARY or groupKey summary headers with id=1)
            // Telegram/TurboTel/WhatsApp post both a group summary header AND individual chat records with exact same text.
            if full_chunk.contains("flags=0x211") || full_chunk.contains("flags=0x210") || full_chunk.contains("FLAG_GROUP_SUMMARY") {
                if full_chunk.contains("groupKey=") && !full_chunk.contains("shortcut=") && (notif_id == "1" || notif_id == "-1") {
                    continue;
                }
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

            let ticker = TICKER_REGEX.captures(&full_chunk).and_then(|c| {
                c.get(1).map(|m| m.as_str().trim().to_string())
            }).filter(|s| s != "null" && !s.is_empty());

            let body = big_text.or(text).or(ticker);

            // Skip notification if both title and body are empty
            if title.is_none() && body.is_none() {
                continue;
            }

            let subtext = SUBTEXT_REGEX.captures(&full_chunk).and_then(|c| {
                c.get(1).or_else(|| c.get(2)).map(|m| m.as_str().trim().to_string())
            }).filter(|s| s != "null" && !s.is_empty());

            let post_time: i64 = WHEN_REGEX
                .captures(&full_chunk)
                .and_then(|c| c.get(1))
                .and_then(|m| m.as_str().parse::<i64>().ok())
                .or_else(|| {
                    POST_TIME_REGEX
                        .captures(&full_chunk)
                        .and_then(|c| c.get(1))
                        .and_then(|m| m.as_str().parse::<i64>().ok())
                })
                .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

            let channel_id = CHANNEL_REGEX
                .captures(&full_chunk)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string());

            // Deduplicate exact content from the same app arriving in the exact same dumpsys snapshot
            let content_key = format!("{}:{}:{}", pkg, title.as_deref().unwrap_or(""), body.as_deref().unwrap_or(""));
            if seen_content_keys.contains(&content_key) {
                continue;
            }
            seen_content_keys.insert(content_key);

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

        items
    }

    /// Formats a package name into a human-readable clean App Name fallback
    pub fn clean_package_fallback(pkg: &str) -> String {
        let parts: Vec<&str> = pkg.split('.').collect();
        let last = parts.last().unwrap_or(&pkg);

        if parts.len() > 1 && (*last == "android" || *last == "app" || *last == "mobile" || *last == "client") {
            let second_last = parts[parts.len() - 2];
            return Self::capitalize_word(second_last);
        }

        Self::capitalize_word(last)
    }

    fn capitalize_word(word: &str) -> String {
        let mut chars = word.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }

    /// Maps package names to readable display names with intelligent fallback
    pub fn resolve_app_name(pkg: &str) -> Option<String> {
        let mut map = HashMap::new();
        // Messaging & Social Apps
        map.insert("com.whatsapp", "WhatsApp");
        map.insert("com.whatsapp.w4b", "WhatsApp Business");
        map.insert("org.telegram.messenger", "Telegram");
        map.insert("org.telegram.plus", "Telegram Plus");
        map.insert("org.thunderdog.challegram", "Telegram X");
        map.insert("com.ellipi.messenger", "TurboTel");
        map.insert("ellipi.messenger", "TurboTel");
        map.insert("org.telegram.BFoundClient", "TurboTel Pro");
        map.insert("org.telegram.messenger.turbo", "TurboTel");
        map.insert("ir.nasim.messenger", "Mobogram");
        map.insert("ir.nasim", "Mobogram");
        map.insert("com.google.android.apps.messaging", "Messages");
        map.insert("com.samsung.android.messaging", "Messages");
        map.insert("com.google.android.gm", "Gmail");
        map.insert("com.microsoft.office.outlook", "Outlook");
        map.insert("com.instagram.android", "Instagram");
        map.insert("com.instagram.barcelona", "Threads");
        map.insert("com.twitter.android", "X (Twitter)");
        map.insert("com.discord", "Discord");
        map.insert("com.slack", "Slack");
        map.insert("com.spotify.music", "Spotify");
        map.insert("org.videolan.vlc", "VLC");
        map.insert("com.mxtech.videoplayer.ad", "MX Player");

        // Iranian Messengers & Popular Apps
        map.insert("ir.eitaa.messenger", "Eitaa");
        map.insert("com.bale.messenger", "Bale");
        map.insert("ir.resaneh.rubika", "Rubika");
        map.insert("ir.medu.shad", "Shad");
        map.insert("com.digikala", "Digikala");
        map.insert("ir.divar", "Divar");
        map.insert("ir.torob", "Torob");
        map.insert("ir.basalam.app", "Basalam");
        map.insert("com.sheypoor.mobile", "Sheypoor");
        map.insert("cab.snapp.passenger", "Snapp");
        map.insert("ir.snapp.passenger", "Snapp");
        map.insert("ir.tapsi.cab", "Tapsi");
        map.insert("ir.mci.ecareapp", "Hamrah Man");
        map.insert("ir.irancell.myirancell", "MyIrancell");
        map.insert("com.myirancell", "MyIrancell");
        map.insert("com.farsitel.bazaar", "Cafe Bazaar");
        map.insert("com.aparat", "Aparat");
        map.insert("com.aparat.filimo", "Filimo");
        map.insert("net.telewebion", "Telewebion");
        map.insert("com.filmju.appmr", "Filmju");

        // Banks & Finance
        map.insert("com.samanpr.blu", "BluBank");
        map.insert("ir.tejaratbank.mobilebank", "Tejarat Bank");
        map.insert("ir.mellatbank.mobilebank", "Mellat Bank");
        map.insert("com.maskanmobilebank", "Maskan Bank");
        map.insert("gold.milli.app", "Milli Gold");
        map.insert("com.ton_keeper", "Tonkeeper");
        map.insert("io.metamask", "MetaMask");

        // System & Tools
        map.insert("com.google.android.youtube", "YouTube");
        map.insert("com.google.android.apps.youtube.music", "YouTube Music");
        map.insert("com.sec.android.app.sbrowser", "Samsung Internet");
        map.insert("com.sec.android.app.notes", "Samsung Notes");
        map.insert("com.sec.android.app.voicenote", "Voice Recorder");
        map.insert("com.sec.android.app.popupcalculator", "Calculator");
        map.insert("ru.zdevs.zarchiver", "ZArchiver");
        map.insert("com.openai.chatgpt", "ChatGPT");
        map.insert("com.deepseek.chat", "DeepSeek");
        map.insert("ai.x.grok", "Grok");
        map.insert("com.duolingo", "Duolingo");
        map.insert("org.lichess.mobileV2", "Lichess");
        map.insert("com.chess", "Chess.com");
        map.insert("com.wireguard.android", "WireGuard");
        map.insert("app.hiddify.com", "Hiddify");
        map.insert("com.v2mmd.ang", "v2rayNG");
        map.insert("com.MahsaNet.MahsaNG", "MahsaNG");
        map.insert("com.github.kr328.clash", "Clash Meta");

        if let Some(&name) = map.get(pkg) {
            Some(name.to_string())
        } else {
            Some(Self::clean_package_fallback(pkg))
        }
    }
}
