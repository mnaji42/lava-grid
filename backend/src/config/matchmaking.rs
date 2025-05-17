/// Matchmaking configuration constants.
/// 
/// This module defines parameters for the matchmaking lobby, such as countdowns,
/// player limits, and timeouts.
pub const COUNTDOWN_DURATION_SECS: u64 = 30; // Countdown before starting a game (in seconds).

/// Minimum number of players required to start a game.
pub const MIN_PLAYERS: usize = 2;

/// Maximum number of players allowed in a game.
pub const MAX_PLAYERS: usize = 3;

/// Time (in seconds) before a player is considered disconnected or inactive.
pub const PLAYER_TIMEOUT: u64 = 60;

/// Time (in seconds) before the game starts after the countdown ends.
/// Used to warn players that the game is about to begin.
pub const PRE_GAME_WARNING_TIME: u64 = 1;