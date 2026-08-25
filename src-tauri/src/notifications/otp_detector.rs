use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OtpResult {
    pub code: String,
    pub confidence: f32, // 0.0 to 1.0
}

lazy_static! {
    // English keywords strictly related to authentication/OTP
    static ref EN_KEYWORDS: Regex = Regex::new(
        r"(?i)\b(verification\s+code|security\s+code|passcode|otp|one[\s-]time\s+password|login\s+code|auth\s+code|pin\s+code|your\s+code\s+is)\b"
    ).unwrap();

    // Persian / Arabic keywords strictly related to authentication/OTP
    static ref FA_KEYWORDS: Regex = Regex::new(
        r"(?i)(کد\s*تایید|کد\s*تأیید|رمز\s*پویا|کد\s*ورود|شماره\s*تایید|کد\s*فعالسازی|رمز\s*یکبار\s*مصرف|کد\s*احراز\s*هویت)"
    ).unwrap();

    // Extract contiguous digits (4 to 8 digits)
    static ref DIGITS_REGEX: Regex = Regex::new(r"\b(\d{4,8})\b").unwrap();
}

pub struct OtpDetector;

impl OtpDetector {
    /// Normalizes Persian (۰-۹) and Arabic (٠-٩) numerals to standard ASCII digits (0-9)
    pub fn normalize_numerals(input: &str) -> String {
        input
            .chars()
            .map(|c| match c {
                // Persian
                '۰' => '0',
                '۱' => '1',
                '۲' => '2',
                '۳' => '3',
                '۴' => '4',
                '۵' => '5',
                '۶' => '6',
                '۷' => '7',
                '۸' => '8',
                '۹' => '9',
                // Arabic-Indic
                '٠' => '0',
                '١' => '1',
                '٢' => '2',
                '٣' => '3',
                '٤' => '4',
                '٥' => '5',
                '٦' => '6',
                '٧' => '7',
                '٨' => '8',
                '٩' => '9',
                _ => c,
            })
            .collect()
    }

    /// Analyzes notification text and returns detected OTP code with confidence score
    pub fn detect(title: Option<&str>, body: Option<&str>) -> Option<OtpResult> {
        let combined_raw = format!("{} {}", title.unwrap_or(""), body.unwrap_or(""));
        let normalized = Self::normalize_numerals(&combined_raw);

        if normalized.trim().is_empty() {
            return None;
        }

        let has_fa_keyword = FA_KEYWORDS.is_match(&normalized);
        let has_en_keyword = EN_KEYWORDS.is_match(&normalized);

        // Find candidate digit groups
        let mut candidates = Vec::new();
        for cap in DIGITS_REGEX.captures_iter(&normalized) {
            if let Some(matched) = cap.get(1) {
                let code = matched.as_str().to_string();

                // Exclude common years (e.g. 2024, 2025, 2026, 1403, 1404)
                if code == "2024" || code == "2025" || code == "2026" || code == "1403" || code == "1404" {
                    continue;
                }

                let mut score: f32 = 0.3; // Baseline confidence

                if has_fa_keyword {
                    score += 0.55;
                }
                if has_en_keyword {
                    score += 0.50;
                }

                // 5 or 6 digits are very common for OTPs
                if code.len() == 5 || code.len() == 6 {
                    score += 0.15;
                }

                let final_score = score.min(1.0);
                candidates.push((code, final_score));
            }
        }

        // Pick highest confidence candidate above 0.70 threshold
        candidates
            .into_iter()
            .filter(|(_, score)| *score >= 0.70)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(code, confidence)| OtpResult { code, confidence })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_english_otp() {
        let res = OtpDetector::detect(Some("Google"), Some("Your verification code is 849201."));
        assert!(res.is_some());
        let otp = res.unwrap();
        assert_eq!(otp.code, "849201");
        assert!(otp.confidence >= 0.8);
    }

    #[test]
    fn test_persian_otp() {
        let res = OtpDetector::detect(Some("بانک ملت"), Some("رمز پویا شما: ۴۸۲۹۱۰ معتبر تا ۲ دقیقه"));
        assert!(res.is_some());
        let otp = res.unwrap();
        assert_eq!(otp.code, "482910");
        assert!(otp.confidence >= 0.85);
    }

    #[test]
    fn test_non_otp_filter() {
        let res = OtpDetector::detect(Some("Shop"), Some("Your total amount is 45000 USD"));
        assert!(res.is_none());
    }
}
