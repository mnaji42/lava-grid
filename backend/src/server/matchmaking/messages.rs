use actix::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use super::types::PlayerInfo;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MatchmakingState {
    pub lobby_players: Vec<PlayerInfo>, // joueurs connectés mais pas encore prêts
    pub ready_players: Vec<PlayerInfo>, // joueurs ayant payé et prêts à jouer
    pub countdown_active: bool,
    pub countdown_remaining: Option<u64>, // en secondes
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "action", content = "data")]
pub enum ClientWsMessage {
    Pay,
    CancelPayment,
    Ping,
}

// Message serveur -> client
#[derive(Message, Serialize, Deserialize, Clone, Debug)]
#[rtype(result = "()")]
#[serde(tag = "action", content = "data")]
pub enum ServerWsMessage {
    UpdateState(MatchmakingState),
    GameStarted {
        game_id: Uuid,
    },
    Error {
        message: String,
    },
}

impl ServerWsMessage {
    pub fn update_state(state: MatchmakingState) -> Self {
        Self::UpdateState(state)
    }
    pub fn game_started(game_id: Uuid) -> Self {
        Self::GameStarted { game_id }
    }
    pub fn error(message: &str) -> Self {
        Self::Error { message: message.to_string() }
    }
}