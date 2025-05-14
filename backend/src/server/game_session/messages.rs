use actix::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use super::session::GameSessionActor;
use crate::game::types::Direction;
use crate::game::state::GameState;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ProcessClientMessage {
    pub msg: ClientAction,
    pub player_id: Uuid,
    pub addr: Addr<GameSessionActor>,
}

#[derive(Serialize, Deserialize)]
pub enum ClientAction {
    Move(Direction),
    Shoot { x: usize, y: usize },
}

#[derive(Message, Clone, Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub struct GameStateUpdate {
    pub state: GameState,
}
