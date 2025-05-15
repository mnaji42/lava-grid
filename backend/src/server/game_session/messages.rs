use actix::prelude::*;
use serde::{Serialize, Deserialize};

use super::session::GameSessionActor;
use crate::game::types::Direction;
use crate::game::state::GameState;
use crate::server::matchmaking::types::WalletAddress;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ProcessClientMessage {
    pub msg: ClientAction,
    pub player_id: WalletAddress,
    pub addr: Addr<GameSessionActor>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientAction {
    Move(Direction),
    Shoot { x: usize, y: usize },
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
    GameStateUpdate { state: GameState, turn_duration: u64 },
    GameEnded { winner: String },
    Error { message: String },
}