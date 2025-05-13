use actix::Addr;
use crate::server::matchmaking::server::MatchmakingServer;
use crate::server::game_session::server::GameSessionManager;

pub struct AppState {
    pub matchmaking_addr: Addr<MatchmakingServer>,
    pub game_session_manager: Addr<GameSessionManager>,
}

impl AppState {
    pub fn new(
        matchmaking_addr: Addr<MatchmakingServer>,
        game_session_manager: Addr<GameSessionManager>
    ) -> Self {
        AppState {
            matchmaking_addr,
            game_session_manager,
        }
    }
}