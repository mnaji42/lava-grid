/// WebSocket session handler for matchmaking lobby.
///
/// This actor manages a single player's connection to the matchmaking lobby,
/// handling join/leave events and relaying client messages (such as payment or cancellation)
/// to the matchmaking server. It also serializes and sends server messages to the client.
use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::borrow::Cow;

use super::messages::{ServerWsMessage, ClientWsMessage};
use super::server::{Join, Leave, Pay, CancelPayment};
use super::types::WalletAddress;

/// Represents a player's WebSocket session in the matchmaking lobby.
pub struct MatchmakingSession {
    pub player_id: WalletAddress,
    pub username: String,
    pub matchmaking_addr: Addr<super::server::MatchmakingServer>,
}

impl Actor for MatchmakingSession {
    type Context = ws::WebsocketContext<Self>;

    /// Called when the session starts. Registers the player in the matchmaking server.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.matchmaking_addr.do_send(Join {
            player_id: self.player_id.clone(),
            addr: ctx.address(),
            username: self.username.clone(),
        });
    }

    /// Called when the session stops. Removes the player from the matchmaking server.
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.matchmaking_addr.do_send(Leave {
            player_id: self.player_id.clone(),
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MatchmakingSession {
    /// Handles incoming WebSocket messages from the client.
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                // Parse the client message as JSON and handle business actions.
                match serde_json::from_str::<ClientWsMessage>(&text) {
                    Ok(ClientWsMessage::Pay) => {
                        // Player wants to pay and become ready.
                        self.matchmaking_addr.do_send(Pay {
                            player_id: self.player_id.clone(),
                        });
                    }
                    Ok(ClientWsMessage::CancelPayment) => {
                        // Player wants to cancel payment and return to lobby.
                        self.matchmaking_addr.do_send(CancelPayment {
                            player_id: self.player_id.clone(),
                        });
                    }
                    Ok(ClientWsMessage::Ping) => {
                        // Ping received; can be ignored or responded to.
                    }
                    Err(_e) => {
                        // Invalid client message format.
                        ctx.text(r#"{"action":"Error","data":{"message":"Invalid client message"}}"#);
                    }
                }
            }
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Close(_)) => ctx.stop(),
            _ => (),
        }
    }
}

impl Handler<ServerWsMessage> for MatchmakingSession {
    type Result = ();

    /// Handles messages sent from the server to this session.
    fn handle(&mut self, msg: ServerWsMessage, ctx: &mut Self::Context) {
        match serde_json::to_string(&msg) {
            Ok(text) => ctx.text(text),
            Err(e) => {
                // Serialization error: notify client and close connection.
                println!("Failed to serialize ServerWsMessage: {}", e);
                ctx.text(r#"{"action":"Error","data":"Internal server error"}"#);
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
    let mut player_id: Option<WalletAddress> = None;
    let mut username = String::new();

    // Parse query parameters for wallet and username.
    for kv in req.query_string().split('&') {
        let mut split = kv.split('=');
        match (split.next(), split.next()) {
            (Some("wallet"), Some(addr)) => {
                // Optionally, address format validation could be added here.
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
            return Ok(HttpResponse::BadRequest().body("Missing wallet address"));
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
        },
        &req,
        stream,
    )
}