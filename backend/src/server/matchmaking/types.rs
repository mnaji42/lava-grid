/// Types used in the matchmaking module.

use serde::{Serialize, Deserialize};

/// Alias for a player's wallet address (unique identifier).
pub type WalletAddress = String;

/// Information about a player in the matchmaking lobby or game.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PlayerInfo {
    /// Unique wallet address of the player.
    pub id: WalletAddress,
    /// Display username.
    pub username: String,
}