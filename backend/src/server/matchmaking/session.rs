
/// WebSocket session handler for a player in the matchmaking lobby.
///
/// Handles incoming client messages (pay, cancel, ping, etc.) and relays server updates.
/// Centralizes error handling and ensures all business logic is executed.

use actix::{Addr, Actor, StreamHandler, Handler, ActorContext, AsyncContext};
use actix_web::{HttpRequest, HttpResponse, web, Error};
use actix_web_actors::ws;
use log::{info, warn, error, debug};

use crate::server::matchmaking::server::{MatchmakingServer, Join, Leave, Pay, CancelPayment};
use crate::server::matchmaking::messages::{ServerWsMessage, ClientWsMessage, SessionKicked};
use crate::server::matchmaking::types::WalletAddress;
use crate::server::ws_error::{ws_error_message, http_error_response, ws_session_kicked_message};
use crate::server::anti_spam::AntiSpamState;

/// Represents a WebSocket session for a player in the matchmaking lobby.
pub struct MatchmakingSession {
    pub player_id: WalletAddress,
    pub username: String,
    pub matchmaking_addr: Addr<MatchmakingServer>,
    pub anti_spam: AntiSpamState,
}

impl Actor for MatchmakingSession {
    type Context = ws::WebsocketContext<Self>;

    /// Register this session with the matchmaking server.
    fn started(&mut self, ctx: &mut Self::Context) {
        info!(
            "[Matchmaking WS] Session started for wallet={} username={}",
            self.player_id, self.username
        );
        // Register this session with the matchmaking server.
        self.matchmaking_addr.do_send(Join {
            player_id: self.player_id.clone(),
            addr: ctx.address(),
            username: self.username.clone(),
        });
    }

    /// Unregister this session from the matchmaking server.
    fn stopped(&mut self, ctx: &mut Self::Context) {
        info!(
            "[Matchmaking WS] Session stopped for wallet={} username={}",
            self.player_id, self.username
        );
        self.matchmaking_addr.do_send(Leave {
            player_id: self.player_id.clone(),
            addr: ctx.address(),
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MatchmakingSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // Anti-spam: record request, close if banned
        if self.anti_spam.record_request(&self.player_id) {
            warn!("[AntiSpam] Banned wallet={} - closing connection", self.player_id);
            ctx.text(ws_error_message(
                "BANNED",
                "You have been banned for spamming. Please try again later.",
                Some(&self.player_id),
            ));
            ctx.close(Some(ws::CloseReason {
                code: ws::CloseCode::Policy,
                description: Some("Banned for spam".into()),
            }));
            ctx.stop();
            return;
        }

        match msg {
            Ok(ws::Message::Text(ref text)) => {
                info!(
                    "[Matchmaking WS] Message received from wallet={}: {}",
                    self.player_id, text
                );
                // Parse the client message as JSON.
                let msg: ClientWsMessage = match serde_json::from_str(text) {
                    Ok(m) => m,
                    Err(e) => {
                        warn!(
                            "[Matchmaking WS] Invalid command from wallet={}: {} | Text: {}",
                            self.player_id, e, text
                        );
                        // Anti-spam: suppress duplicate errors
                        if self.anti_spam.should_send_error("INVALID_ACTION", &self.player_id) {
                            if self.anti_spam.record_response(&self.player_id) {
                                ctx.text(ws_error_message(
                                    "BANNED",
                                    "You have been banned for spamming. Please try again later.",
                                    Some(&self.player_id),
                                ));
                                ctx.close(Some(ws::CloseReason {
                                    code: ws::CloseCode::Policy,
                                    description: Some("Banned for spam".into()),
                                }));
                                ctx.stop();
                                return;
                            }
                            ctx.text(ws_error_message(
                                "INVALID_ACTION",
                                "Invalid command",
                                Some(&self.player_id),
                            ));
                        }
                        return;
                    }
                };
                debug!(
                    "[Matchmaking WS] Successfully parsed client message for wallet={}: {:?}",
                    self.player_id, msg
                );
                // Handle the parsed client message.
                match msg {
                    ClientWsMessage::Pay => {
                        self.matchmaking_addr.do_send(Pay {
                            player_id: self.player_id.clone(),
                            addr: ctx.address(),
                        });
                        self.anti_spam.reset_on_valid_action();
                    }
                    ClientWsMessage::CancelPayment => {
                        self.matchmaking_addr.do_send(CancelPayment {
                            player_id: self.player_id.clone(),
                            addr: ctx.address(),
                        });
                        self.anti_spam.reset_on_valid_action();
                    }
                    ClientWsMessage::Ping => {
                        debug!("[Matchmaking WS] Received Ping from wallet={}", self.player_id);
                        // Optionally, respond or ignore.
                    }
                }
            }
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Close(_)) => {
                info!("[Matchmaking WS] Connection closed: wallet={}", self.player_id);
                ctx.stop();
            }
            Ok(other) => {
                warn!(
                    "[Matchmaking WS] Ignored WebSocket message from wallet={}: {:?}",
                    self.player_id, other
                );
            }
            Err(e) => {
                error!(
                    "[Matchmaking WS] WebSocket error: wallet={} err={:?}",
                    self.player_id, e
                );
                if self.anti_spam.should_send_error("WS_PROTOCOL_ERROR", &self.player_id) {
                    if self.anti_spam.record_response(&self.player_id) {
                        ctx.text(ws_error_message(
                            "BANNED",
                            "You have been banned for spamming. Please try again later.",
                            Some(&self.player_id),
                        ));
                        ctx.close(Some(ws::CloseReason {
                            code: ws::CloseCode::Policy,
                            description: Some("Banned for spam".into()),
                        }));
                        ctx.stop();
                        return;
                    }
                    ctx.text(ws_error_message(
                        "WS_PROTOCOL_ERROR",
                        "WebSocket protocol error",
                        Some(&self.player_id),
                    ));
                }
                ctx.stop();
            }
        }
    }
}

impl Handler<SessionKicked> for MatchmakingSession {
    type Result = ();

    /// Handles the session being kicked from the server.
    fn handle(&mut self, _msg: SessionKicked, ctx: &mut Self::Context) -> Self::Result {
        info!("[Matchmaking WS] Session kicked: wallet={}", self.player_id);
        ctx.text(ws_session_kicked_message(Some(&self.player_id)));
        ctx.stop();
    }
}

impl Handler<ServerWsMessage> for MatchmakingSession {
    type Result = ();

    /// Handles messages sent from the server to this session.
    fn handle(&mut self, msg: ServerWsMessage, ctx: &mut Self::Context) {
        match serde_json::to_string(&msg) {
            Ok(text) => {
                if self.anti_spam.record_response(&self.player_id) {
                    ctx.text(ws_error_message(
                        "BANNED",
                        "You have been banned for spamming. Please try again later.",
                        Some(&self.player_id),
                    ));
                    ctx.close(Some(ws::CloseReason {
                        code: ws::CloseCode::Policy,
                        description: Some("Banned for spam".into()),
                    }));
                    ctx.stop();
                    return;
                }
                ctx.text(text)
            },
            Err(e) => {
                error!(
                    "[Matchmaking WS] Failed to serialize ServerWsMessage for wallet={}: {}",
                    self.player_id, e
                );
                if self.anti_spam.should_send_error("SERIALIZATION_ERROR", &self.player_id) {
                    if self.anti_spam.record_response(&self.player_id) {
                        ctx.text(ws_error_message(
                            "BANNED",
                            "You have been banned for spamming. Please try again later.",
                            Some(&self.player_id),
                        ));
                        ctx.close(Some(ws::CloseReason {
                            code: ws::CloseCode::Policy,
                            description: Some("Banned for spam".into()),
                        }));
                        ctx.stop();
                        return;
                    }
                    ctx.text(ws_error_message(
                        "SERIALIZATION_ERROR",
                        "Internal server error",
                        Some(&self.player_id),
                    ));
                }
                ctx.close(Some(ws::CloseReason {
                    code: ws::CloseCode::Error,
                    description: Some("Internal server error".into()),
                }));
                ctx.stop();
            }
        }
    }
}

/// WebSocket endpoint for matchmaking lobby.
///
/// Expects query parameters: `wallet` (player address), `username` (optional).
/// If username is missing, a default is generated from the wallet address.
pub async fn ws_matchmaking(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<crate::server::state::AppState>,
) -> Result<HttpResponse, Error> {
    use std::borrow::Cow;
    let mut player_id: Option<WalletAddress> = None;
    let mut username = String::new();

    // Parse query parameters for wallet and username.
    for kv in req.query_string().split('&') {
        let mut split = kv.split('=');
        match (split.next(), split.next()) {
            (Some("wallet"), Some(addr)) => {
                player_id = Some(addr.to_string());
            }
            (Some("username"), Some(name)) => {
                username = urlencoding::decode(name)
                    .unwrap_or_else(|_| Cow::Borrowed(""))
                    .into_owned();
            }
            _ => {}
        }
    }

    // Reject connection if wallet address is missing.
    let player_id = match player_id {
        Some(addr) if !addr.is_empty() => addr,
        _ => {
            warn!("[Matchmaking WS] Connection refused: missing wallet address");
            return Ok(http_error_response(
                "MISSING_WALLET",
                "Missing wallet address",
                None,
                actix_web::http::StatusCode::BAD_REQUEST,
            ));
        }
    };

    // If username is empty, generate a default one.
    if username.is_empty() {
        username = format!("Joueur_{}", &player_id[..6]);
    }

    ws::start(
        MatchmakingSession {
            player_id,
            username,
            matchmaking_addr: data.matchmaking_addr.clone(),
            anti_spam: AntiSpamState::new(),
        },
        &req,
        stream,
    )
}
