// src/server/state.rs

//! Application state for the backend server.
//!
//! Holds references to the main actor addresses (matchmaking and game session managers).
//! Used to share state between HTTP/WebSocket handlers and the actor system.

use actix::Addr;
use crate::server::matchmaking::server::MatchmakingServer;
use crate::server::game_session::server::GameSessionManager;

/// Shared application state, injected into HTTP/WebSocket handlers.
pub struct AppState {
    /// Address of the matchmaking server actor (handles lobby, payments, readiness).
    pub matchmaking_addr: Addr<MatchmakingServer>,
    /// Address of the game session manager actor (handles game orchestration).
    pub game_session_manager: Addr<GameSessionManager>,
}

impl AppState {
    /// Create a new AppState with the given actor addresses.
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