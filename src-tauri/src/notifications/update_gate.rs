use lazy_static::lazy_static;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Per-notification-key state used to collapse high-frequency updates
/// (e.g. download progress bars that refresh every second).
struct KeyState {
    /// Hash of the last payload we actually forwarded (save + emit)
    last_payload_hash: u64,
    /// Hash of the last content we showed a desktop toast for
    last_toasted_hash: u64,
    last_toasted_at: Option<Instant>,
}

impl Default for KeyState {
    fn default() -> Self {
        Self {
            last_payload_hash: 0,
            last_toasted_hash: 0,
            last_toasted_at: None,
        }
    }
}

lazy_static! {
    static ref GATE: std::sync::Mutex<HashMap<String, KeyState>> =
        std::sync::Mutex::new(HashMap::new());
}

const MIN_TOAST_INTERVAL: Duration = Duration::from_secs(20);

fn fnv1a(data: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in data.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Decides which notification updates are worth acting on. Both the ADB
/// (logcat/dumpsys) pipeline and the companion-app WebSocket pipeline funnel
/// events through this gate so behaviour stays consistent.
pub struct UpdateGate;

impl UpdateGate {
    /// True if this exact payload differs from what was last forwarded for
    /// the key. Identical repeat payloads (common with chatty progress bars)
    /// are swallowed so they never reach the DB, frontend or toasts.
    pub fn should_forward(key: &str, signature: &str) -> bool {
        let hash = fnv1a(signature);
        let mut gate = GATE.lock().unwrap();
        let state = gate.entry(key.to_string()).or_default();
        if state.last_payload_hash == hash {
            return false;
        }
        state.last_payload_hash = hash;
        true
    }

    /// Toast policy:
    ///   - First time we see a key                      -> toast (new message)
    ///   - OTP codes                                    -> always toast
    ///   - Content changed AND >= 20s since last toast  -> toast (real follow-up)
    /// Everything else (per-second progress ticks) is silent. The feed still
    /// receives every distinct update via should_forward.
    pub fn should_show_toast(key: &str, signature: &str, is_otp: bool) -> bool {
        let hash = fnv1a(signature);
        let mut gate = GATE.lock().unwrap();
        let state = gate.entry(key.to_string()).or_default();

        let now = Instant::now();
        let allowed = if is_otp {
            // Verification codes always surface immediately
            true
        } else if state.last_toasted_at.is_none() && state.last_toasted_hash == 0 {
            // Never toasted this key before
            true
        } else {
            match state.last_toasted_at {
                Some(last) => {
                    now.duration_since(last) >= MIN_TOAST_INTERVAL && state.last_toasted_hash != hash
                }
                None => true,
            }
        };

        if allowed {
            state.last_toasted_at = Some(now);
            state.last_toasted_hash = hash;
        }
        allowed
    }

    /// Drops tracking state once a notification is removed from the phone.
    pub fn forget(key: &str) {
        GATE.lock().unwrap().remove(key);
    }
}