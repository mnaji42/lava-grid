use actix::prelude::*;
use actix::fut;
use uuid::Uuid;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::types::{PlayerInfo};
use super::messages::{ServerWsMessage, MatchmakingState};
use super::session::MatchmakingSession;
use crate::config::matchmaking::{MIN_PLAYERS, COUNTDOWN_DURATION, MAX_PLAYERS, PRE_GAME_WARNING_TIME};

type SessionAddr = Addr<MatchmakingSession>;

pub struct MatchmakingServer {
    sessions: HashMap<Uuid, (PlayerInfo, SessionAddr)>,
    countdown: Option<CountdownHandles>,
}

struct CountdownHandles {
    warning_handle: SpawnHandle,
    game_start_handle: SpawnHandle,
    start_time: Instant,
}

impl MatchmakingServer {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            countdown: None,
        }
    }

    fn broadcast(&self, msg: ServerWsMessage) {
        for (_, (_player_info, addr)) in &self.sessions {
            addr.do_send(msg.clone());
        }
    }

    fn get_state(&self) -> MatchmakingState {
        let time_remaining = self.countdown.as_ref().map_or(COUNTDOWN_DURATION, |h| {
            let elapsed = h.start_time.elapsed().as_secs();
            COUNTDOWN_DURATION.saturating_sub(elapsed)
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
        if self.countdown.is_none() && self.sessions.len() >= MIN_PLAYERS {

            let warning_handle = ctx.run_later(
                Duration::from_secs(COUNTDOWN_DURATION - PRE_GAME_WARNING_TIME), 
                |act, _| {
                    act.broadcast(ServerWsMessage::update_state(act.get_state()));
                }
            );

            let game_start_handle = ctx.run_later(
                Duration::from_secs(COUNTDOWN_DURATION), 
                |act, ctx| {
                    act.broadcast(ServerWsMessage::game_started(Uuid::new_v4()));
                    ctx.stop();
                }
            );

            self.countdown = Some(CountdownHandles {
                warning_handle,
                game_start_handle,
                start_time: Instant::now(),
            });
        }
    }
}

impl Actor for MatchmakingServer {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub player_id: Uuid,
    pub addr: SessionAddr,
    pub username: String,
}

impl Handler<Join> for MatchmakingServer {
    type Result = ();

    fn handle(&mut self, msg: Join, ctx: &mut Self::Context) -> Self::Result {
        if self.sessions.len() >= MAX_PLAYERS {
            // TODO gerer le cas si un joueur essaye de rejoindre un matchmaking et qu'il y a deja max_player
            return;
        }
        let player_info = PlayerInfo {
            id: msg.player_id,
            username: msg.username,
        };

        self.sessions.insert(msg.player_id, (player_info, msg.addr));
        self.start_countdown(ctx);
        let state = self.get_state();
        self.broadcast(ServerWsMessage::player_join(state));
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Leave {
    pub player_id: Uuid,
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