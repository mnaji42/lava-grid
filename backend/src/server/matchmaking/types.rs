use serde::{Serialize, Deserialize};

pub type WalletAddress = String;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PlayerInfo {
    pub id: WalletAddress,
    pub username: String,
}