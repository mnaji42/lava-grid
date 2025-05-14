use actix::{Addr, Actor, StreamHandler, AsyncContext, Handler};
use actix_web::{HttpRequest, HttpResponse, web, Error, error};
use actix_web_actors::ws;
use uuid::Uuid;
use log::{info, warn, error, debug};

use crate::server::game_session::server::{GameSession, IsPlayerInGame, GetGameSession, RegisterSession, UnregisterSession};
use crate::server::game_session::messages::{ProcessClientMessage, GameStateUpdate, ClientAction, GameWsMessage};
use crate::server::matchmaking::types::WalletAddress;

pub struct GameSessionActor {
    pub game_id: Uuid,
    pub player_id: WalletAddress,
    pub is_player: bool,
    pub session_addr: Addr<GameSession>,
}

impl Actor for GameSessionActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // info!(
        //     "[WS] Connexion: wallet={} game_id={} is_player={}",
        //     self.player_id, self.game_id, self.is_player
        // );
        self.session_addr.do_send(RegisterSession {
            wallet: self.player_id.clone(),
            addr: ctx.address(),
            is_player: self.is_player,
        });
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        // info!(
        //     "[WS] Déconnexion: wallet={} game_id={}",
        //     self.player_id, self.game_id
        // );
        self.session_addr.do_send(UnregisterSession {
            wallet: self.player_id.clone(),
            is_player: self.is_player,
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for GameSessionActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(ref text)) => {
                info!(
                    "[WS] Message reçu de wallet={} (is_player={}): {}",
                    self.player_id, self.is_player, text
                );
                if !self.is_player {
                    warn!(
                        "[WS] Tentative de commande par spectateur: wallet={}",
                        self.player_id
                    );
                    ctx.text(r#"{"error":"Spectators cannot send commands"}"#);
                    return;
                }
                let msg: ClientAction = match serde_json::from_str(text) {
                    Ok(m) => m,
                    Err(e) => {
                        warn!(
                            "[WS] Commande invalide reçue de wallet={}: {}",
                            self.player_id, e
                        );
                        ctx.text(r#"{"error":"Invalid command"}"#);
                        return;
                    }
                };
                // info!(
                //     "[WS] Action client traitée: wallet={}",
                //     self.player_id
                // );
                self.session_addr.do_send(ProcessClientMessage {
                    msg,
                    player_id: self.player_id.clone(),
                    addr: ctx.address(),
                });
            }
            Ok(ws::Message::Ping(_)) => {
                debug!("[WS] Ping reçu de wallet={}", self.player_id);
            }
            Ok(ws::Message::Close(_)) => {
                info!("[WS] Fermeture de la connexion: wallet={}", self.player_id);
            }
            Ok(other) => {
                debug!("[WS] Message WebSocket ignoré: {:?}", other);
            }
            Err(e) => {
                error!("[WS] Erreur WebSocket: wallet={} err={:?}", self.player_id, e);
            }
        }
    }
}

impl Handler<GameStateUpdate> for GameSessionActor {
    type Result = ();
    fn handle(&mut self, msg: GameStateUpdate, ctx: &mut Self::Context) -> Self::Result {
        debug!(
            "[WS] Envoi de GameStateUpdate à wallet={} (is_player={}): turn={} players={:?}",
            self.player_id,
            self.is_player,
            msg.state.turn,
            msg.state.players.iter().map(|p| (p.id, p.pos, p.is_alive)).collect::<Vec<_>>()
        );
        let ws_msg = GameWsMessage::GameStateUpdate { state: msg.state };
        match serde_json::to_string(&ws_msg) {
            Ok(text) => ctx.text(text),
            Err(e) => {
                error!(
                    "[WS] Erreur de sérialisation GameStateUpdate pour wallet={}: {}",
                    self.player_id, e
                );
                ctx.text(r#"{"action":"Error","data":{"message":"Failed to serialize game state"}}"#)
            }
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
        Err(_) => {
            warn!("[WS] game_id invalide reçu: {}", game_id_str);
            return Ok(HttpResponse::BadRequest().body("Invalid game_id"));
        }
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
            warn!("[WS] Connexion refusée: wallet manquant pour game_id={}", game_id);
            return Ok(HttpResponse::BadRequest().body("Missing wallet address"));
        }
    };

    // info!(
    //     "[WS] Tentative de connexion: wallet={} game_id={}",
    //     player_id, game_id
    // );

    // Vérifier si le joueur fait partie de la partie
    let is_player = data.game_session_manager
        .send(IsPlayerInGame { game_id, player_id: player_id.clone() })
        .await
        .map_err(error::ErrorInternalServerError)?
        .map_err(error::ErrorBadRequest)?;

    // info!(
    //     "[WS] Rôle déterminé pour wallet={} dans game_id={}: is_player={}",
    //     player_id, game_id, is_player
    // );

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