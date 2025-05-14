
use actix::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use super::types::PlayerInfo;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MatchmakingState {
    pub players: Vec<PlayerInfo>,
    pub countdown_active: bool,
    pub time_remaining: u64,
}


// Message serveur -> client
#[derive(Message, Serialize, Deserialize, Clone, Debug)]
#[rtype(result = "()")]
#[serde(tag = "action", content = "data")]
pub enum ServerWsMessage {
    PlayerJoin(MatchmakingState),
    PlayerLeave(MatchmakingState),
    UpdateState(MatchmakingState),
    GameStarted {
        game_id: Uuid,
    },
    Error {
        message: String,
    },
}

// Implémentation des constructeurs de messages
impl ServerWsMessage {
    pub fn player_join(state: MatchmakingState) -> Self {
        Self::PlayerJoin(state)
    }

    pub fn player_leave(state: MatchmakingState) -> Self {
        Self::PlayerLeave(state)
    }

    pub fn update_state(state: MatchmakingState) -> Self {
        Self::UpdateState(state)
    }

    pub fn game_started(game_id: Uuid) -> Self {
        Self::GameStarted { game_id }
    }

    pub fn error(message: &str) -> Self {
        Self::Error {
            message: message.to_string(),
        }
    }
}
