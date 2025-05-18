use std::collections::HashMap;
use std::time::{Instant, Duration};
use log::{warn, info};

use crate::config::anti_spam::{MAX_RESPONSES_PER_SECOND, MAX_REQUESTS_PER_SECOND, BAN_DURATION_SECONDS};

/// Tracks anti-spam state for a single session (player or spectator).
pub struct AntiSpamState {
    // Last error code sent (for suppression)
    last_error_code: Option<String>,
    // Timestamp of last reset (for per-second counters)
    last_tick: Instant,
    // Number of responses sent in the current second
    responses_this_tick: u32,
    // Number of requests received in the current second
    requests_this_tick: u32,
    // Ban state
    banned_until: Option<Instant>,
}

impl AntiSpamState {
    pub fn new() -> Self {
        Self {
            last_error_code: None,
            last_tick: Instant::now(),
            responses_this_tick: 0,
            requests_this_tick: 0,
            banned_until: None,
        }
    }

    /// Call at the start of every incoming request (message).
    /// Returns true if the session is currently banned.
    pub fn record_request(&mut self, wallet: &str) -> bool {
        self.tick();
        self.requests_this_tick += 1;
        if self.requests_this_tick > MAX_REQUESTS_PER_SECOND {
            self.ban(wallet, "Too many requests per second");
            return true;
        }
        self.is_banned()
    }

    /// Call at the start of every outgoing response (including errors).
    /// Returns true if the session is currently banned.
    pub fn record_response(&mut self, wallet: &str) -> bool {
        self.tick();
        self.responses_this_tick += 1;
        if self.responses_this_tick > MAX_RESPONSES_PER_SECOND {
            self.ban(wallet, "Too many responses per second");
            return true;
        }
        self.is_banned()
    }

    /// Call when sending an error. Returns true if the error should be sent (not suppressed).
    pub fn should_send_error(&mut self, error_code: &str, wallet: &str) -> bool {
        if let Some(last) = &self.last_error_code {
            if last == error_code {
                // Suppress duplicate error
                warn!("[AntiSpam] Suppressed duplicate error '{}' for wallet={}", error_code, wallet);
                return false;
            }
        }
        self.last_error_code = Some(error_code.to_string());
        true
    }

    /// Call when a valid action is performed (state-changing, not error).
    pub fn reset_on_valid_action(&mut self) {
        self.last_error_code = None;
        // Optionally, could reset counters here if desired
    }

    /// Call to reset error suppression (e.g., at start of new turn or phase).
    pub fn reset_error_suppression(&mut self) {
        self.last_error_code = None;
    }

    /// Returns true if the session is currently banned.
    pub fn is_banned(&self) -> bool {
        if let Some(until) = self.banned_until {
            Instant::now() < until
        } else {
            false
        }
    }

    /// Returns the ban expiry time, if banned.
    pub fn ban_expiry(&self) -> Option<Instant> {
        self.banned_until
    }

    /// Returns the remaining ban duration in seconds, or 0 if not banned.
    pub fn ban_remaining_secs(&self, wallet: &str) -> u64 {
        if let Some(until) = self.banned_until {
            let now = Instant::now();
            if until > now {
                return (until - now).as_secs();
            }
        }
        0
    }

    /// Ban the session for BAN_DURATION_SECONDS.
    fn ban(&mut self, wallet: &str, reason: &str) {
        let until = Instant::now() + Duration::from_secs(BAN_DURATION_SECONDS);
        self.banned_until = Some(until);
        warn!("[AntiSpam] Banned wallet={} until {:?} for reason: {}", wallet, until, reason);
    }

    /// Reset per-second counters if a new second has started.
    fn tick(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_tick) >= Duration::from_secs(1) {
            self.last_tick = now;
            self.responses_this_tick = 0;
            self.requests_this_tick = 0;
        }
    }
}
