use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PlayerInfo {
    pub id: Uuid,
    pub username: String,
}
