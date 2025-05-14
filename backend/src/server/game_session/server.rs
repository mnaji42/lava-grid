use actix::prelude::*;
use uuid::Uuid;
use std::collections::HashMap;
use actix::MessageResult;

use crate::game::state::GameState;
use crate::server::matchmaking::types::PlayerInfo;
use crate::server::matchmaking::server::CreateGame;
use crate::server::game_session::session::GameSessionActor;
use crate::server::game_session::messages::{GameStateUpdate, ProcessClientMessage};


pub struct GameSession {
    pub game_id: Uuid,
    pub players: Vec<PlayerInfo>,
    pub spectators: HashMap<Uuid, Addr<GameSessionActor>>,
    pub game_state: GameState,
}

impl Actor for GameSession {
    type Context = Context<Self>;
}

pub struct GameSessionManager {
    sessions: HashMap<Uuid, Addr<GameSession>>,
}

impl GameSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn create_game(&mut self, players: Vec<PlayerInfo>) -> Uuid {
        let game_id = Uuid::new_v4();
        let game_state = GameState::new(5, 5, players.len());
        
        let session = GameSession {
            game_id,
            players: players.clone(),
            spectators: HashMap::new(),
            game_state,
        }.start();
        
        self.sessions.insert(game_id, session);
        game_id
    }
}

impl Actor for GameSessionManager {
    type Context = Context<Self>;
}

impl Handler<CreateGame> for GameSessionManager {
    type Result = MessageResult<CreateGame>;

    fn handle(&mut self, msg: CreateGame, _: &mut Context<Self>) -> Self::Result {
        MessageResult(self.create_game(msg.players))
    }
}

#[derive(Message)]
#[rtype(result = "Result<Addr<GameSession>, String>")]
pub struct GetGameSession {
    pub game_id: Uuid,
}

impl Handler<GetGameSession> for GameSessionManager {
    type Result = Result<Addr<GameSession>, String>;

    fn handle(&mut self, msg: GetGameSession, _: &mut Context<Self>) -> Self::Result {
        self.sessions.get(&msg.game_id)
            .cloned()
            .ok_or_else(|| "Game session not found".to_string())
    }
}

impl GameSession {
    pub fn send_state(&self) {
        let state = self.game_state.clone();
        self.spectators.values().for_each(|addr| {
            addr.do_send(GameStateUpdate { state: state.clone() });
        });
    }
}

#[derive(Message)]
#[rtype(result = "Result<bool, String>")]
pub struct IsPlayerInGame {
    pub game_id: Uuid,
    pub player_id: Uuid,
}

impl Handler<IsPlayerInGame> for GameSessionManager {
    type Result = Result<bool, String>;

    fn handle(&mut self, msg: IsPlayerInGame, _: &mut Context<Self>) -> Self::Result {
        self.sessions.get(&msg.game_id)
            .map(|session_addr| {
                // Vérifier si le joueur fait partie de la session
                session_addr.try_send(IsPlayer(msg.player_id))
                    .map(|_| true)
                    .unwrap_or(false)
            })
            .ok_or_else(|| "Game session not found".to_string())
    }
}

#[derive(Message)]
#[rtype(result = "bool")]
pub struct IsPlayer(pub Uuid);

impl Handler<IsPlayer> for GameSession {
    type Result = bool;

    fn handle(&mut self, msg: IsPlayer, _: &mut Context<Self>) -> Self::Result {
        self.players.iter().any(|p| p.id == msg.0)
    }
}

impl Handler<ProcessClientMessage> for GameSession {
    type Result = ();

    fn handle(&mut self, msg: ProcessClientMessage, _ctx: &mut Context<Self>) -> Self::Result {
        // Ici tu traites l'action du joueur (msg.msg)
        // Par exemple, tu modifies self.game_state selon l'action
        // Puis tu broadcast le nouvel état à tous les spectateurs

        // TODO: Appliquer l'action sur self.game_state
        // (ex: move, shoot, etc.)

        // Pour l’instant, on peut juste faire un broadcast de l’état courant
        self.send_state();
    }
}