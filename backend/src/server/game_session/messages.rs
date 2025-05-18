//! Messages and types for game session actors and WebSocket protocol.
//!
//! Defines all messages exchanged between the game session actors, as well as the protocol
//! for client-server communication during a game.

use actix::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use super::session::GameSessionActor;
use crate::game::types::{Direction, GameMode};
use crate::game::state::GameState;
use crate::server::matchmaking::types::{WalletAddress, PlayerInfo};
use crate::server::game_session::GameSession;

/// Message to register a pending game (sent by matchmaking when a group is ready).
#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterPendingGame {
    pub game_id: Uuid,
    pub players: Vec<PlayerInfo>,
}

/// Message to request creation or retrieval of a GameSession for a given game_id.
/// Used when a client connects to a game WebSocket.
#[derive(Message)]
#[rtype(result = "Result<Addr<GameSession>, String>")]
pub struct EnsureGameSession {
    pub game_id: Uuid,
    pub mode: Option<GameMode>,
}

/// Message sent by a player to perform an action (move or shoot).
#[derive(Message)]
#[rtype(result = "()")]
pub struct ProcessClientMessage {
    pub msg: PlayerAction,
    pub player_id: WalletAddress,
    pub addr: Addr<GameSessionActor>,
}

/// Player action sent by the client (move or shoot).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PlayerAction {
    Move(Direction),
    Shoot { x: usize, y: usize },
}

/// WebSocket messages sent from client to server during a game session.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "action", content = "data")]
pub enum GameClientWsMessage {
    /// Move in a direction.
    Move(Direction),
    /// Shoot at a tile.
    Shoot { x: usize, y: usize },
    /// Vote for a game mode.
    GameModeVote { mode: GameMode },
}

/// Message sent when a player votes for a game mode.
#[derive(Message)]
#[rtype(result = "()")]
pub struct GameModeVote {
    pub player_id: WalletAddress,
    pub mode: GameMode,
}

/// Data sent to all players at the start of the game or when the pre-game phase is refreshed.
#[derive(Message, Clone, Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub struct GamePreGameData {
    pub modes: Vec<GameMode>,
    pub deadline_secs: u64,
    pub players: Vec<PlayerInfo>,
    pub grid_row: usize,
    pub grid_col: usize,
}

/// Notification sent to all players when a player votes for a mode.
#[derive(Message, Clone, Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub struct GameModeVoteUpdate {
    pub player_id: WalletAddress,
    pub mode: GameMode,
}

/// Notification of the chosen mode and the player who was selected to decide.
#[derive(Message, Clone, Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub struct GameModeChosen {
    pub mode: GameMode,
    pub chosen_by: WalletAddress,
}

/// Game state update sent to all players after each turn.
#[derive(Message, Clone, Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub struct GameStateUpdate {
    pub state: GameState,
    pub turn_duration: u64,
}

/// Message to kick a session (unicity violation).
#[derive(Message)]
#[rtype(result = "()")]
pub struct SessionKicked {
    pub reason: String,
}

/// WebSocket messages sent from server to client during a game session.
#[derive(Message, Serialize, Deserialize, Clone, Debug)]
#[rtype(result = "()")]
#[serde(tag = "action", content = "data")]
pub enum GameWsMessage {
    /// Initial game state and mode.
    GameInit { state: GameState, mode: GameMode },
    /// Game state update after a turn.
    GameStateUpdate { state: GameState, turn_duration: u64 },
    /// Game ended, with winner.
    GameEnded { winner: String },
    /// Error message.
    Error { message: String },
    /// Session kicked notification.
    SessionKicked { reason: String },
    /// Pre-game data (mode choice, players, deadline).
    GamePreGameData(GamePreGameData),
    /// Notification of a mode vote.
    GameModeVoteUpdate(GameModeVoteUpdate),
    /// Notification of the chosen mode.
    GameModeChosen(GameModeChosen),
}
