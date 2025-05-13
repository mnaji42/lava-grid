use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use uuid::Uuid;
use std::borrow::Cow;

use super::messages::ServerWsMessage;
use super::server::{Join, Leave};

pub struct MatchmakingSession {
    pub player_id: Uuid,
    pub username: String,
    pub matchmaking_addr: Addr<super::server::MatchmakingServer>,
}

impl Actor for MatchmakingSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.matchmaking_addr.do_send(Join {
            player_id: self.player_id,
            addr: ctx.address(),
            username: self.username.clone(),
        });
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.matchmaking_addr.do_send(Leave {
            player_id: self.player_id,
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MatchmakingSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
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
    let mut player_id = Uuid::new_v4();
    let mut username = String::new();

    for kv in req.query_string().split('&') {
        let mut split = kv.split('=');
        match (split.next(), split.next()) {
            (Some("player_id"), Some(id)) => {
                if let Ok(uuid) = Uuid::parse_str(id) {
                    player_id = uuid;
                }
            }
            (Some("username"), Some(name)) => {
                username = urlencoding::decode(name)
                    .unwrap_or_else(|_| Cow::Borrowed(""))
                    .into_owned();
            }
            _ => {}
        }
    }

    if username.is_empty() {
        username = format!("Joueur_{}", &player_id.to_string()[..4]);
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