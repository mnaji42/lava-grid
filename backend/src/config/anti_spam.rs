/// Configuration for anti-spam and anti-flood protection.
/// All values are in seconds or counts per second.
pub const MAX_RESPONSES_PER_SECOND: u32 = 5;
pub const MAX_REQUESTS_PER_SECOND: u32 = 30;
pub const BAN_DURATION_SECONDS: u64 = 300;