use actix::prelude::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

use super::types::{PlayerInfo, WalletAddress};
use super::messages::{ServerWsMessage, MatchmakingState};
use super::session::MatchmakingSession;
use crate::config::matchmaking::{MIN_PLAYERS, COUNTDOWN_DURATION, MAX_PLAYERS, PRE_GAME_WARNING_TIME};
use crate::server::game_session::server::GameSessionManager;

type SessionAddr = Addr<MatchmakingSession>;

pub struct MatchmakingServer {
    sessions: HashMap<WalletAddress, (PlayerInfo, SessionAddr)>,
    countdown: Option<CountdownHandles>,
    game_session_manager: Addr<GameSessionManager>,
}

struct CountdownHandles {
    warning_handle: SpawnHandle,
    game_start_handle: SpawnHandle,
    start_time: Instant,
}

#[derive(Message)]
#[rtype(result = "Uuid")]
pub struct CreateGame {
    pub players: Vec<PlayerInfo>,
}

impl MatchmakingServer {
    pub fn new(game_session_manager: Addr<GameSessionManager>) -> Self {
        Self {
            sessions: HashMap::new(),
            countdown: None,
            game_session_manager,
        }
    }

    fn broadcast(&self, msg: ServerWsMessage) {
        for (_, (_player_info, addr)) in &self.sessions {
            addr.do_send(msg.clone());
        }
    }

    fn get_state(&self) -> MatchmakingState {
        let time_remaining = self.countdown.as_ref().map_or(COUNTDOWN_DURATION, |h| {
            COUNTDOWN_DURATION.saturating_sub(h.start_time.elapsed().as_secs())
        });

        MatchmakingState {
            players: self.sessions.values()
                .map(|(player_info, _)| player_info.clone())
                .collect(),
            countdown_active: self.countdown.is_some(),
            time_remaining,
        }
    }

    fn start_countdown(&mut self, ctx: &mut Context<Self>) {
        let warning_handle = ctx.run_later(
            Duration::from_secs(COUNTDOWN_DURATION - PRE_GAME_WARNING_TIME), 
            |act, _| {
                act.broadcast(ServerWsMessage::update_state(act.get_state()));
            }
        );

        let game_start_handle = ctx.run_later(
            Duration::from_secs(COUNTDOWN_DURATION),
            |act, ctx| act.start_game(ctx),
        );

        self.countdown = Some(CountdownHandles {
            warning_handle,
            game_start_handle,
            start_time: Instant::now(),
        });
    }

    fn start_game(&mut self, ctx: &mut Context<Self>) {
        let players = self.sessions.values()
            .map(|(player_info, _)| player_info.clone())
            .collect();

        self.game_session_manager
            .send(CreateGame { players })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(game_id) => {
                        act.broadcast(ServerWsMessage::game_started(game_id));
                    }
                    Err(_e) => {
                        act.broadcast(ServerWsMessage::error("Erreur création partie"));
                    }
                }

                act.sessions.clear();

                if let Some(handles) = act.countdown.take() {
                    ctx.cancel_future(handles.warning_handle);
                    ctx.cancel_future(handles.game_start_handle);
                }

                fut::ready(())
            })
            .wait(ctx);
    }
}

impl Actor for MatchmakingServer {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub player_id: WalletAddress,
    pub addr: SessionAddr,
    pub username: String,
}

impl Handler<Join> for MatchmakingServer {
    type Result = ();

    fn handle(&mut self, msg: Join, ctx: &mut Self::Context) -> Self::Result {
        if self.sessions.len() >= MAX_PLAYERS {
            // TODO: handle max player error
            return;
        }
        let player_info = PlayerInfo {
            id: msg.player_id.clone(),
            username: msg.username,
        };

        self.sessions.insert(msg.player_id.clone(), (player_info, msg.addr));

        if self.sessions.len() >= MAX_PLAYERS {
            self.start_game(ctx);
        } else {
            if self.countdown.is_none() && self.sessions.len() >= MIN_PLAYERS {
                self.start_countdown(ctx);
            }
            let state = self.get_state();
            self.broadcast(ServerWsMessage::player_join(state));
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Leave {
    pub player_id: WalletAddress,
}

impl Handler<Leave> for MatchmakingServer {
    type Result = ();

    fn handle(&mut self, msg: Leave, ctx: &mut Self::Context) -> Self::Result {
        if self.sessions.remove(&msg.player_id).is_some() {
            if self.sessions.len() < MIN_PLAYERS {
                if let Some(handles) = self.countdown.take() {
                    ctx.cancel_future(handles.warning_handle);
                    ctx.cancel_future(handles.game_start_handle);
                }
            }

            let state = self.get_state();
            self.broadcast(ServerWsMessage::player_leave(state));
        }
    }
}