use actix::{Addr, Actor, StreamHandler, AsyncContext, Handler};
use actix_web::{HttpRequest, HttpResponse, web, Error, error};
use actix_web_actors::ws;
use uuid::Uuid;
use log::{info, warn, error, debug};

use crate::server::game_session::server::{GameSession, UnregisterSession, RegisterSession};
use crate::server::game_session::messages::{GamePreGameData, GameModeChosen, ProcessClientMessage, GameStateUpdate, PlayerAction, GameWsMessage, EnsureGameSession, GameModeVoteUpdate, GameClientWsMessage};
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
        self.session_addr.do_send(RegisterSession {
            wallet: self.player_id.clone(),
            addr: ctx.address(),
            is_player: self.is_player,
        });
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
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
                debug!(
                    "[WS] Tentative de parsing du message client pour wallet={}: {}",
                    self.player_id, text
                );
                let msg: GameClientWsMessage = match serde_json::from_str(text) {
                    Ok(m) => m,
                    Err(e) => {
                        warn!(
                            "[WS] Commande invalide reçue de wallet={}: {} | Texte reçu: {}",
                            self.player_id, e, text
                        );
                        ctx.text(r#"{"error":"Invalid command"}"#);
                        return;
                    }
                };
                debug!(
                    "[WS] Message client parsé avec succès pour wallet={}: {:?}",
                    self.player_id, msg
                );
                match msg {
                    GameClientWsMessage::Move(dir) => {
                        self.session_addr.do_send(ProcessClientMessage {
                            msg: PlayerAction::Move(dir),
                            player_id: self.player_id.clone(),
                            addr: ctx.address(),
                        });
                    }
                    GameClientWsMessage::Shoot { x, y } => {
                        self.session_addr.do_send(ProcessClientMessage {
                            msg: PlayerAction::Shoot { x, y },
                            player_id: self.player_id.clone(),
                            addr: ctx.address(),
                        });
                    }
                    GameClientWsMessage::GameModeVote { mode } => {
                        // Ici, on envoie un message spécifique pour le vote de mode
                        self.session_addr.do_send(crate::server::game_session::messages::GameModeVote {
                            player_id: self.player_id.clone(),
                            mode,
                        });
                    }
                    // Ajoute d'autres variantes ici si besoin
                }
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



impl Handler<GamePreGameData> for GameSessionActor {
    type Result = ();
    fn handle(&mut self, msg: GamePreGameData, ctx: &mut Self::Context) -> Self::Result {
        let ws_msg = GameWsMessage::GamePreGameData(msg);
        match serde_json::to_string(&ws_msg) {
            Ok(text) => ctx.text(text),
            Err(_e) => ctx.text(r#"{"action":"Error","data":{"message":"Failed to serialize available game modes"}}"#),
        }
    }
}

impl Handler<GameModeChosen> for GameSessionActor {
    type Result = ();
    fn handle(&mut self, msg: GameModeChosen, ctx: &mut Self::Context) -> Self::Result {
        let ws_msg = GameWsMessage::GameModeChosen(msg);
        match serde_json::to_string(&ws_msg) {
            Ok(text) => ctx.text(text),
            Err(_e) => ctx.text(r#"{"action":"Error","data":{"message":"Failed to serialize chosen mode"}}"#),
        }
    }
}

impl Handler<GameModeVoteUpdate> for GameSessionActor {
    type Result = ();
    fn handle(&mut self, msg: GameModeVoteUpdate, ctx: &mut Self::Context) -> Self::Result {
        let ws_msg = GameWsMessage::GameModeVoteUpdate(msg);
        match serde_json::to_string(&ws_msg) {
            Ok(text) => ctx.text(text),
            Err(_e) => ctx.text(r#"{"action":"Error","data":{"message":"Failed to serialize vote update"}}"#),
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
            msg.state.players.iter().map(|p| (p.id.clone(), p.pos, p.is_alive)).collect::<Vec<_>>()
        );
        let ws_msg = GameWsMessage::GameStateUpdate { state: msg.state, turn_duration: msg.turn_duration };
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

    // Nouvelle logique : on demande la création (ou récupération) de la GameSession à la connexion
    let session_addr = data.game_session_manager
        .send(EnsureGameSession { game_id, mode: None }) // mode: None, à gérer plus tard
        .await
        .map_err(error::ErrorInternalServerError)?
        .map_err(error::ErrorBadRequest)?;

    // On vérifie si le joueur fait partie de la partie (logique existante)
    let is_player = session_addr
        .send(crate::server::game_session::server::IsPlayer(player_id.clone()))
        .await
        .unwrap_or(false);

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