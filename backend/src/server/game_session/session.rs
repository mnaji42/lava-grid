//! WebSocket session handler for a player or spectator in a game session.
//!
//! This actor manages a single WebSocket connection to a game session, handling
//! incoming client messages (actions, votes) and relaying server updates.

use actix::{Addr, Actor, StreamHandler, Handler, ActorContext};
use actix_web::{HttpRequest, HttpResponse, web, Error};
use actix_web_actors::ws;
use uuid::Uuid;
use log::{info, warn, error, debug};
use actix::AsyncContext;

use crate::server::game_session::server::{GameSession, UnregisterSession, RegisterSession};
use crate::server::game_session::messages::{
    GamePreGameData, GameModeChosen, ProcessClientMessage, GameStateUpdate, PlayerAction,
    GameWsMessage, EnsureGameSession, GameModeVoteUpdate, GameClientWsMessage, GameModeVote,
    SessionKicked
};
use crate::server::matchmaking::types::WalletAddress;
use crate::server::ws_error::{ws_error_message, http_error_response, ws_session_kicked_message};

/// Represents a WebSocket session for a player or spectator in a game.
pub struct GameSessionActor {
    pub game_id: Uuid,
    pub player_id: WalletAddress,
    pub is_player: bool,
    pub session_addr: Addr<GameSession>,
}

impl Actor for GameSessionActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Register this session with the GameSession actor.
        self.session_addr.do_send(RegisterSession {
            wallet: self.player_id.clone(),
            addr: ctx.address(),
            is_player: self.is_player,
        });
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        // Unregister this session from the GameSession actor.
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
                    "[WS] Message received from wallet={} (is_player={}): {}",
                    self.player_id, self.is_player, text
                );
                // Only allow players (not spectators) to send commands.
                if !self.is_player {
                    warn!(
                        "[WS] Command attempt by spectator: wallet={}",
                        self.player_id
                    );
                    ctx.text(ws_error_message(
                        "SPECTATOR_COMMAND",
                        "Spectators cannot send commands",
                        Some(&self.player_id),
                    ));
                    return;
                }
                debug!(
                    "[WS] Attempting to parse client message for wallet={}: {}",
                    self.player_id, text
                );
                // Parse the client message as JSON.
                let msg: GameClientWsMessage = match serde_json::from_str(text) {
                    Ok(m) => m,
                    Err(e) => {
                        warn!(
                            "[WS] Invalid command received from wallet={}: {} | Text: {}",
                            self.player_id, e, text
                        );
                        ctx.text(ws_error_message(
                            "INVALID_ACTION",
                            "Invalid command",
                            Some(&self.player_id),
                        ));
                        return;
                    }
                };
                debug!(
                    "[WS] Successfully parsed client message for wallet={}: {:?}",
                    self.player_id, msg
                );
                // Handle the parsed client message.
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
                        // Forward the mode vote to the session.
                        self.session_addr.do_send(GameModeVote {
                            player_id: self.player_id.clone(),
                            mode,
                        });
                    }
                    // Add other variants here if needed.
                }
            }
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Close(_)) => {
                info!("[WS] Connection closed: wallet={}", self.player_id);
                ctx.stop();
            }
            Ok(other) => {
                debug!("[WS] Ignored WebSocket message: {:?}", other);
            }
            Err(e) => {
                error!("[WS] WebSocket error: wallet={} err={:?}", self.player_id, e);
                ctx.text(ws_error_message(
                    "WS_PROTOCOL_ERROR",
                    "WebSocket protocol error",
                    Some(&self.player_id),
                ));
                ctx.stop();
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
            Err(_e) => ctx.text(ws_error_message(
                "SERIALIZATION_ERROR",
                "Failed to serialize available game modes",
                Some(&self.player_id),
            )),
        }
    }
}

impl Handler<GameModeChosen> for GameSessionActor {
    type Result = ();
    fn handle(&mut self, msg: GameModeChosen, ctx: &mut Self::Context) -> Self::Result {
        let ws_msg = GameWsMessage::GameModeChosen(msg);
        match serde_json::to_string(&ws_msg) {
            Ok(text) => ctx.text(text),
            Err(_e) => ctx.text(ws_error_message(
                "SERIALIZATION_ERROR",
                "Failed to serialize chosen mode",
                Some(&self.player_id),
            )),
        }
    }
}

impl Handler<GameModeVoteUpdate> for GameSessionActor {
    type Result = ();
    fn handle(&mut self, msg: GameModeVoteUpdate, ctx: &mut Self::Context) -> Self::Result {
        let ws_msg = GameWsMessage::GameModeVoteUpdate(msg);
        match serde_json::to_string(&ws_msg) {
            Ok(text) => ctx.text(text),
            Err(_e) => ctx.text(ws_error_message(
                "SERIALIZATION_ERROR",
                "Failed to serialize vote update",
                Some(&self.player_id),
            )),
        }
    }
}

impl Handler<GameStateUpdate> for GameSessionActor {
    type Result = ();
    fn handle(&mut self, msg: GameStateUpdate, ctx: &mut Self::Context) -> Self::Result {
        debug!(
            "[WS] Sending GameStateUpdate to wallet={} (is_player={}): turn={} players={:?}",
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
                    "[WS] Serialization error GameStateUpdate for wallet={}: {}",
                    self.player_id, e
                );
                ctx.text(ws_error_message(
                    "SERIALIZATION_ERROR",
                    "Failed to serialize game state",
                    Some(&self.player_id),
                ))
            }
        }
    }
}

impl Handler<SessionKicked> for GameSessionActor {
    type Result = ();

    fn handle(&mut self, _msg: SessionKicked, ctx: &mut Self::Context) -> Self::Result {
        info!("[WS] Session kicked: wallet={}", self.player_id);
        ctx.text(ws_session_kicked_message(Some(&self.player_id)));
        ctx.stop();
    }
}

/// WebSocket endpoint for joining a game session.
/// Expects path parameter: `game_id` and query parameter: `wallet`.
pub async fn ws_game(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<crate::server::state::AppState>,
) -> Result<HttpResponse, Error> {
    let game_id_str = req.match_info().get("game_id").unwrap().to_string();
    let game_id = match Uuid::parse_str(&game_id_str) {
        Ok(uuid) => uuid,
        Err(_) => {
            warn!("[WS] Invalid game_id received: {}", game_id_str);
            return Ok(http_error_response(
                "INVALID_GAME_ID",
                "Invalid game_id",
                Some(&game_id_str),
                actix_web::http::StatusCode::BAD_REQUEST,
            ));
        }
    };

    // Extract wallet address from query parameters.
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
            warn!("[WS] Connection refused: missing wallet for game_id={}", game_id);
            return Ok(http_error_response(
                "MISSING_WALLET",
                "Missing wallet address",
                Some(&game_id.to_string()),
                actix_web::http::StatusCode::BAD_REQUEST,
            ));
        }
    };

    // Ensure the game session exists or create it.
    let session_addr = match data
        .game_session_manager
        .send(EnsureGameSession {
            game_id,
            mode: None,
        })
        .await
    {
        Ok(Ok(addr)) => addr,
        Ok(Err(e)) => {
            error!(
                "[WS] Failed to ensure game session for game_id={}: {}",
                game_id, e
            );
            return Ok(http_error_response(
                "GAME_SESSION_ERROR",
                &e,
                Some(&game_id.to_string()),
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
        Err(e) => {
            error!(
                "[WS] Mailbox error when ensuring game session for game_id={}: {}",
                game_id, e
            );
            return Ok(http_error_response(
                "MAILBOX_ERROR",
                "Internal server error",
                Some(&game_id.to_string()),
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    ws::start(
        GameSessionActor {
            game_id,
            player_id: player_id.clone(),
            is_player: true, // TODO: handle spectators if needed
            session_addr,
        },
        &req,
        stream,
    )
}
