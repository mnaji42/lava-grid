use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::borrow::Cow;

use super::messages::{ServerWsMessage, ClientWsMessage};
use super::server::{Join, Leave, Pay, CancelPayment};
use super::types::WalletAddress;

pub struct MatchmakingSession {
    pub player_id: WalletAddress,
    pub username: String,
    pub matchmaking_addr: Addr<super::server::MatchmakingServer>,
}

impl Actor for MatchmakingSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.matchmaking_addr.do_send(Join {
            player_id: self.player_id.clone(),
            addr: ctx.address(),
            username: self.username.clone(),
        });
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.matchmaking_addr.do_send(Leave {
            player_id: self.player_id.clone(),
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MatchmakingSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                // Parse le message JSON du client
                match serde_json::from_str::<ClientWsMessage>(&text) {
                    Ok(ClientWsMessage::Pay) => {
                        self.matchmaking_addr.do_send(Pay {
                            player_id: self.player_id.clone(),
                        });
                    }
                    Ok(ClientWsMessage::CancelPayment) => {
                        self.matchmaking_addr.do_send(CancelPayment {
                            player_id: self.player_id.clone(),
                        });
                    }
                    Ok(ClientWsMessage::Ping) => {
                        // Optionnel : on pourrait répondre par un pong ou ignorer
                    }
                    Err(_e) => {
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

    fn handle(&mut self, msg: ServerWsMessage, ctx: &mut Self::Context) {
        match serde_json::to_string(&msg) {
            Ok(text) => ctx.text(text),
            Err(e) => {
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

pub async fn ws_matchmaking(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<crate::server::state::AppState>,
) -> Result<HttpResponse, Error> {
    let mut player_id: Option<WalletAddress> = None;
    let mut username = String::new();

    for kv in req.query_string().split('&') {
        let mut split = kv.split('=');
        match (split.next(), split.next()) {
            (Some("wallet"), Some(addr)) => {
                // TODO: Optionally validate address format (0x...)
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

    let player_id = match player_id {
        Some(addr) if !addr.is_empty() => addr,
        _ => {
            // Reject connection if wallet not provided
            return Ok(HttpResponse::BadRequest().body("Missing wallet address"));
        }
    };

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