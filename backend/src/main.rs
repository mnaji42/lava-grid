use actix::Actor;
use actix_web::{web, App, HttpServer};
use server::matchmaking::server::MatchmakingServer;
use server::game_session::server::GameSessionManager;

pub mod config;
mod server;
mod game;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let game_session_manager = GameSessionManager::new().start();
    let manager_clone = game_session_manager.clone(); 
    
    let matchmaking_addr = MatchmakingServer::new(manager_clone).start();
    
    let state = web::Data::new(server::state::AppState::new(
        matchmaking_addr,
        game_session_manager,
    ));

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