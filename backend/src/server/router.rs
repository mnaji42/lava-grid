use actix_web::web;
use crate::server::matchmaking::session::ws_matchmaking;
use crate::server::game_session::session::ws_game;

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