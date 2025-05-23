/// Messages exchanged between client and server in the matchmaking lobby.

use actix::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use super::types::PlayerInfo;

/// State of the matchmaking lobby, sent to clients.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MatchmakingState {
    /// Players connected but not yet ready.
    pub lobby_players: Vec<PlayerInfo>,
    /// Players who have paid and are ready to play.
    pub ready_players: Vec<PlayerInfo>,
    /// Whether a countdown is active for starting a game.
    pub countdown_active: bool,
    /// Remaining countdown time in seconds, if active.
    pub countdown_remaining: Option<u64>,
}

/// Messages sent from client to server over WebSocket.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "action", content = "data")]
pub enum ClientWsMessage {
    /// Player wants to pay and become ready.
    Pay,
    /// Player wants to cancel payment and return to lobby.
    CancelPayment,
    /// Ping (keepalive or latency check).
    Ping,
}

/// Message to notify that a session has been kicked (e.g., due to being replaced).
#[derive(Message)]
#[rtype(result = "()")]
pub struct SessionKicked {
    pub reason: String,
}


/// Messages sent from server to client over WebSocket.
#[derive(Message, Serialize, Deserialize, Clone, Debug)]
#[rtype(result = "()")]
#[serde(tag = "action", content = "data")]
pub enum ServerWsMessage {
    /// Update the current state of the lobby.
    UpdateState(MatchmakingState),
    /// Notify the client that a game has started, with the assigned game ID.
    GameStarted {
        game_id: Uuid,
    },
    /// Notify the client of an error.
    Error {
        message: String,
    },
    /// Notify the client that their session has been kicked.
    SessionKicked {
        reason: String,
    },
}

