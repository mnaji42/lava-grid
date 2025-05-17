use actix::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use super::session::GameSessionActor;
use crate::game::types::{Direction, GameMode};
use crate::game::state::GameState;
use crate::server::matchmaking::types::{WalletAddress, PlayerInfo};
use crate::server::game_session::GameSession;

/// Message pour enregistrer une partie en attente (appelé par le matchmaking)
#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterPendingGame {
    pub game_id: Uuid,
    pub players: Vec<PlayerInfo>,
}

/// Message pour demander la création (ou récupération) d'une GameSession à la connexion WebSocket
#[derive(Message)]
#[rtype(result = "Result<Addr<GameSession>, String>")]
pub struct EnsureGameSession {
    pub game_id: Uuid,
    pub mode: Option<GameMode>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ProcessClientMessage {
    pub msg: PlayerAction,
    pub player_id: WalletAddress,
    pub addr: Addr<GameSessionActor>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PlayerAction {
    Move(Direction),
    Shoot { x: usize, y: usize },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "action", content = "data")]
pub enum GameClientWsMessage {
    Move(Direction),
    Shoot { x: usize, y: usize },
    GameModeVote { mode: GameMode },
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct GameModeVote {
    pub player_id: WalletAddress,
    pub mode: GameMode,
}

/// Nouveau message envoyé à chaque joueur à la connexion ou lors du refresh de la phase de pré-game
#[derive(Message, Clone, Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub struct GamePreGameData {
    pub modes: Vec<GameMode>,
    pub deadline_secs: u64,
    pub players: Vec<PlayerInfo>,
    pub grid_row: usize,
    pub grid_col: usize,
}

/// Notification à tous les joueurs lorsqu'un joueur a voté
#[derive(Message, Clone, Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub struct GameModeVoteUpdate {
    pub player_id: WalletAddress,
    pub mode: GameMode,
}

/// Notification du mode choisi et du joueur tiré au sort
#[derive(Message, Clone, Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub struct GameModeChosen {
    pub mode: GameMode,
    pub chosen_by: WalletAddress,
}

#[derive(Message, Clone, Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub struct GameStateUpdate {
    pub state: GameState,
    pub turn_duration: u64,
}

#[derive(Message, Serialize, Deserialize, Clone, Debug)]
#[rtype(result = "()")]
#[serde(tag = "action", content = "data")]
pub enum GameWsMessage {
    GameInit { state: GameState, mode: GameMode },
    GameStateUpdate { state: GameState, turn_duration: u64 },
    GameEnded { winner: String },
    Error { message: String },
    GamePreGameData(GamePreGameData),
    GameModeVoteUpdate(GameModeVoteUpdate),
    GameModeChosen(GameModeChosen),
}