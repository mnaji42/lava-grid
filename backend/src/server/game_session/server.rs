use actix::prelude::*;
use std::collections::HashMap;
use actix::MessageResult;
use uuid::Uuid;
use log::{info, debug};

use crate::game::state::GameState;
use crate::server::matchmaking::types::{PlayerInfo, WalletAddress};
use crate::server::matchmaking::server::CreateGame;
use crate::server::game_session::session::GameSessionActor;
use crate::server::game_session::messages::{GameStateUpdate, ProcessClientMessage};

pub struct GameSession {
    pub game_id: Uuid,
    pub player_infos: Vec<PlayerInfo>,
    pub players: HashMap<WalletAddress, Addr<GameSessionActor>>,
    pub spectators: HashMap<WalletAddress, Addr<GameSessionActor>>,
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
        // info!(
        //     "[GameSessionManager] Création d'une nouvelle partie: game_id={} joueurs={:?}",
        //     game_id,
        //     players.iter().map(|p| (&p.id, &p.username)).collect::<Vec<_>>()
        // );
        let game_state = GameState::new(5, 5, players.len());

        let session = GameSession {
            game_id,
            player_infos: players.clone(),
            players: HashMap::new(),
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
        // info!(
        //     "[GameSessionManager] Reçu CreateGame pour joueurs={:?}",
        //     msg.players.iter().map(|p| (&p.id, &p.username)).collect::<Vec<_>>()
        // );
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
        // debug!(
        //     "[GameSessionManager] GetGameSession: game_id={}",
        //     msg.game_id
        // );
        self.sessions.get(&msg.game_id)
            .cloned()
            .ok_or_else(|| "Game session not found".to_string())
    }
}

impl GameSession {
    pub fn send_state(&self) {
        debug!( // Celui la je le garde
            "[GameSession] Broadcast GameState: game_id={} turn={} players={:?}",
            self.game_id,
            self.game_state.turn,
            self.game_state.players.iter().map(|p| &p.id).collect::<Vec<_>>()
        );
        let state = self.game_state.clone();
        // Broadcast à tous les joueurs connectés
        for addr in self.players.values().chain(self.spectators.values()) {
            addr.do_send(GameStateUpdate { state: state.clone() });
        }
    }
}

#[derive(Message)]
#[rtype(result = "Result<bool, String>")]
pub struct IsPlayerInGame {
    pub game_id: Uuid,
    pub player_id: WalletAddress,
}

impl Handler<IsPlayerInGame> for GameSessionManager {
    type Result = Result<bool, String>;

    fn handle(&mut self, msg: IsPlayerInGame, _: &mut Context<Self>) -> Self::Result {
        // debug!(
        //     "[GameSessionManager] Vérification si wallet={} est joueur dans game_id={}",
        //     msg.player_id, msg.game_id
        // );
        self.sessions.get(&msg.game_id)
            .map(|session_addr| {
                session_addr.try_send(IsPlayer(msg.player_id.clone()))
                    .map(|_| true)
                    .unwrap_or(false)
            })
            .ok_or_else(|| "Game session not found".to_string())
    }
}

#[derive(Message)]
#[rtype(result = "bool")]
pub struct IsPlayer(pub WalletAddress);

impl Handler<IsPlayer> for GameSession {
    type Result = bool;

    fn handle(&mut self, msg: IsPlayer, _: &mut Context<Self>) -> Self::Result {
        let is_player = self.player_infos.iter().any(|p| p.id == msg.0);
        // debug!(
        //     "[GameSession] IsPlayer: wallet={} -> {}",
        //     msg.0, is_player
        // );
        is_player
    }
}

impl Handler<ProcessClientMessage> for GameSession {
    type Result = ();

    fn handle(&mut self, msg: ProcessClientMessage, _ctx: &mut Context<Self>) -> Self::Result {
        // info!(
        //     "[GameSession] Action client reçue: wallet={}",
        //     msg.player_id
        // );
        // TODO: Apply action to game_state, broadcast new state
        self.send_state();
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterSession {
    pub wallet: WalletAddress,
    pub addr: Addr<GameSessionActor>,
    pub is_player: bool,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UnregisterSession {
    pub wallet: WalletAddress,
    pub is_player: bool,
}


impl Handler<RegisterSession> for GameSession {
    type Result = ();

    fn handle(&mut self, msg: RegisterSession, _: &mut Context<Self>) -> Self::Result {
        // info!(
        //     "[GameSession] RegisterSession: wallet={} game_id={} is_player={}",
        //     msg.wallet, self.game_id, msg.is_player
        // );
        if msg.is_player {
            self.players.insert(msg.wallet.clone(), msg.addr.clone());
        } else {
            self.spectators.insert(msg.wallet.clone(), msg.addr.clone());
        }
        let state = self.game_state.clone();
        msg.addr.do_send(GameStateUpdate { state });
    }
}


impl Handler<UnregisterSession> for GameSession {
    type Result = ();

    fn handle(&mut self, msg: UnregisterSession, _: &mut Context<Self>) -> Self::Result {
        // info!(
        //     "[GameSession] UnregisterSession: wallet={} game_id={} is_player={}",
        //     msg.wallet, self.game_id, msg.is_player
        // );
        if msg.is_player {
            self.players.remove(&msg.wallet);
        } else {
            self.spectators.remove(&msg.wallet);
        }
    }
}