//! HTTP and WebSocket routing configuration.
//!
//! Defines the main endpoints for matchmaking and game sessions.
//! Each endpoint is handled by a dedicated WebSocket actor.

use actix_web::web;
use crate::server::matchmaking::session::ws_matchmaking;
use crate::server::game_session::session::ws_game;

/// Configure the application's HTTP/WebSocket routes.
///
/// Each route is handled by its respective actor, which manages the connection lifecycle
/// and business logic for that context.
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/ws/matchmaking")
            .to(ws_matchmaking)
    )
    .service(
        web::resource("/ws/game/{game_id}")
            .to(ws_game)
    );
}