use actix_web::web;
use crate::server::matchmaking::session::ws_matchmaking;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/ws/matchmaking")
            .to(ws_matchmaking)
    );
}