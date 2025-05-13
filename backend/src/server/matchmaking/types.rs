
use actix::{Message, Recipient};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PlayerInfo {
    pub id: Uuid,
    pub username: String,
}

#[derive(Clone, Debug, Message, Serialize)]
#[rtype(result = "()")]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
    PlayerList(Vec<PlayerInfo>),
    CountdownStart { seconds: u8 },
    CountdownCancel,
    GameStarted {
        game_id: String,
        players: Vec<PlayerInfo>,
    },
}
