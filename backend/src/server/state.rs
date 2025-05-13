use actix::Addr;
use crate::server::matchmaking::server::MatchmakingServer;

pub struct AppState {
    pub matchmaking_addr: Addr<MatchmakingServer>,
}

impl AppState {
    pub fn new(matchmaking_addr: Addr<MatchmakingServer>) -> Self {
        AppState { matchmaking_addr }
    }
}