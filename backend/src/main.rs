use actix::Actor;
use actix_web::{web, App, HttpServer};
use server::matchmaking::server::MatchmakingServer;

pub mod config;
mod server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matchmaking_addr = MatchmakingServer::new().start();
    let state = web::Data::new(server::state::AppState::new(matchmaking_addr));

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