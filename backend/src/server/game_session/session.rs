use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use actix_web_actors::ws::{Message, ProtocolError};
use actix::StreamHandler;
use uuid::Uuid;

impl StreamHandler<Result<Message, ProtocolError>> for GameSession {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        // Handle WebSocket messages here
    }
}

pub struct GameSession {
    pub game_id: Uuid,
    // Ajouter d'autres champs nécessaires
}

impl Actor for GameSession {
    type Context = ws::WebsocketContext<Self>;
}

pub async fn ws_game(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<crate::server::state::AppState>,
) -> Result<HttpResponse, Error> {
    let game_id = req.match_info().get("game_id").unwrap();
    let game_id = Uuid::parse_str(game_id).map_err(|e| {
        actix_web::error::ErrorBadRequest(format!("Invalid game ID: {}", e))
    })?;

    ws::start(
        GameSession {
            game_id,
        },
        &req,
        stream,
    )
}