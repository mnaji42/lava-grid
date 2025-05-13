use actix::prelude::*;
use uuid::Uuid;
use std::collections::HashMap;
use actix::MessageResult;

use crate::game::state::GameState;
use crate::server::matchmaking::types::PlayerInfo;
use crate::server::matchmaking::server::CreateGame;

pub struct GameSession {
    pub game_id: Uuid,
    pub players: Vec<PlayerInfo>,
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