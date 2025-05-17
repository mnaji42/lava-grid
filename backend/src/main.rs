//! Main entry point for the backend server.
//!
//! Initializes the actor system, configures application state, and launches the HTTP server
//! with WebSocket endpoints for matchmaking and game sessions.

use actix::Actor;
use actix_web::{web, App, HttpServer};
use server::matchmaking::server::MatchmakingServer;
use server::game_session::server::GameSessionManager;

pub mod config;
mod server;
mod game;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger from environment variable (default to info level).
    env_logger::init();

    // Start the GameSessionManager actor (handles all game sessions).
    let game_session_manager = GameSessionManager::new().start();

    // Start the MatchmakingServer actor (handles lobby, payments, readiness).
    let matchmaking_addr = MatchmakingServer::new(game_session_manager.clone()).start();
    
    // Shared application state for HTTP/WebSocket handlers.
    let state = web::Data::new(server::state::AppState::new(
        matchmaking_addr,
        game_session_manager,
    ));

    // Start the HTTP server with WebSocket endpoints.
    HttpServer::new(move || {
        App::new()
            .wrap(
                actix_web::middleware::DefaultHeaders::new()
                    .add(("Access-Control-Allow-Origin", "*"))
                    .add(("Access-Control-Allow-Headers", "*"))
            )
            .app_data(state.clone())
            .configure(crate::server::router::config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}