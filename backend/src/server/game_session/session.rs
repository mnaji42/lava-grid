use actix::{Addr, Actor, StreamHandler, AsyncContext, Handler};
use actix_web::{HttpRequest, HttpResponse, web, Error, error};
use actix_web_actors::ws;
use uuid::Uuid;

use crate::server::game_session::server::{GameSession, IsPlayerInGame, GetGameSession};
use crate::server::game_session::messages::{ProcessClientMessage, GameStateUpdate, ClientAction};
use crate::server::matchmaking::types::WalletAddress;

pub struct GameSessionActor {
    pub game_id: Uuid,
    pub player_id: WalletAddress,
    pub is_player: bool,
    pub session_addr: Addr<GameSession>,
}

impl Actor for GameSessionActor {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for GameSessionActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                if !self.is_player {
                    ctx.text(r#"{"error":"Spectators cannot send commands"}"#);
                    return;
                }
                let msg: ClientAction = match serde_json::from_str(&text) {
                    Ok(m) => m,
                    Err(_) => {
                        ctx.text(r#"{"error":"Invalid command"}"#);
                        return;
                    }
                };
                self.session_addr.do_send(ProcessClientMessage {
                    msg,
                    player_id: self.player_id.clone(),
                    addr: ctx.address(),
                });
            }
            _ => (),
        }
    }
}

impl Handler<GameStateUpdate> for GameSessionActor {
    type Result = ();
    fn handle(&mut self, msg: GameStateUpdate, ctx: &mut Self::Context) -> Self::Result {
        match serde_json::to_string(&msg) {
            Ok(text) => ctx.text(text),
            Err(_) => ctx.text(r#"{"error":"Failed to serialize game state"}"#),
        }
    }
}

pub async fn ws_game(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<crate::server::state::AppState>,
) -> Result<HttpResponse, Error> {
    let game_id_str = req.match_info().get("game_id").unwrap().to_string();
    let game_id = match Uuid::parse_str(&game_id_str) {
        Ok(uuid) => uuid,
        Err(_) => return Ok(HttpResponse::BadRequest().body("Invalid game_id")),
    };

    // Récupérer le wallet depuis les query parameters
    let mut player_id: Option<WalletAddress> = None;
    for kv in req.query_string().split('&') {
        let mut split = kv.split('=');
        match (split.next(), split.next()) {
            (Some("wallet"), Some(addr)) => {
                player_id = Some(addr.to_string());
            }
            _ => {}
        }
    }
    let player_id = match player_id {
        Some(addr) if !addr.is_empty() => addr,
        _ => {
            return Ok(HttpResponse::BadRequest().body("Missing wallet address"));
        }
    };

    // Vérifier si le joueur fait partie de la partie
    let is_player = data.game_session_manager
        .send(IsPlayerInGame { game_id, player_id: player_id.clone() })
        .await
        .map_err(error::ErrorInternalServerError)?
        .map_err(error::ErrorBadRequest)?;

    let session_addr = data.game_session_manager
        .send(GetGameSession { game_id })
        .await
        .map_err(error::ErrorInternalServerError)?
        .map_err(error::ErrorBadRequest)?;

    ws::start(
        GameSessionActor {
            game_id,
            player_id,
            is_player,
            session_addr,
        },
        &req,
        stream,
    )
}