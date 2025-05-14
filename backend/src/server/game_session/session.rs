use actix::{Addr, Actor, StreamHandler, AsyncContext, Handler};
use actix_web::{HttpRequest, HttpResponse, web, Error, error};
use actix_web_actors::ws;
use uuid::Uuid;

use crate::server::game_session::server::{GameSession, IsPlayerInGame, GetGameSession};
use crate::server::game_session::messages::{ProcessClientMessage, GameStateUpdate, ClientAction};

pub struct GameSessionActor {
    pub game_id: Uuid,
    pub player_id: Uuid,
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
                // Désérialise le message du client
                let msg: ClientAction = match serde_json::from_str(&text) {
                    Ok(m) => m,
                    Err(_) => {
                        ctx.text(r#"{"error":"Invalid command"}"#);
                        return;
                    }
                };
                self.session_addr.do_send(ProcessClientMessage {
                    msg,
                    player_id: self.player_id,
                    addr: ctx.address(),
                });
            }
            _ => (),
        }
    }
}

// Handler pour recevoir les updates d'état de jeu
impl Handler<GameStateUpdate> for GameSessionActor {
    type Result = ();
    fn handle(&mut self, msg: GameStateUpdate, ctx: &mut Self::Context) -> Self::Result {
        // Envoie l'état du jeu au client via WebSocket
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
    let game_id = req.match_info().get("game_id").unwrap();
    let game_id = Uuid::parse_str(game_id).map_err(|e| error::ErrorBadRequest(e))?;
    
    // Récupérer le player_id depuis les query parameters
    let player_id = req.query_string()
        .split('&')
        .find(|s| s.starts_with("player_id="))
        .and_then(|s| Uuid::parse_str(s.split('=').nth(1).unwrap_or("")).ok())
        .unwrap_or_else(Uuid::new_v4); // Générer un ID pour les spectateurs

    println!("Player id: {}", player_id)
    // Vérifier si le joueur fait partie de la partie
    let is_player = data.game_session_manager
        .send(IsPlayerInGame { game_id, player_id })
        .await
        .map_err(error::ErrorInternalServerError)?
        .map_err(error::ErrorBadRequest)?;

    println!("Is player: {}", is_player)
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