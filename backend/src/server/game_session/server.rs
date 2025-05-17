//! Game session management and orchestration.
//!
//! This module defines the actors responsible for managing game sessions, including
//! player registration, game state progression, mode voting, and turn resolution.

use actix::prelude::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;
use log::{info, warn, debug};

use crate::game::state::GameState;
use crate::server::matchmaking::types::{PlayerInfo, WalletAddress};
use crate::server::game_session::session::GameSessionActor;
use crate::config::game::{TURN_DURATION, MODE_CHOICE_DURATION, GRID_ROW, GRID_COL};
use crate::game::types::{GameMode, Direction};
use crate::server::game_session::messages::{
    GameStateUpdate, ProcessClientMessage, PlayerAction, RegisterPendingGame, EnsureGameSession,
    GameModeChosen, GamePreGameData, GameModeVote, GameModeVoteUpdate
};
use rand::prelude::IteratorRandom;

/// Stores pending games waiting for session creation.
pub struct PendingGames {
    pub pending: HashMap<Uuid, Vec<PlayerInfo>>,
}

impl PendingGames {
    pub fn new() -> Self {
        Self { pending: HashMap::new() }
    }
    pub fn insert(&mut self, game_id: Uuid, players: Vec<PlayerInfo>) {
        self.pending.insert(game_id, players);
    }
    pub fn take(&mut self, game_id: &Uuid) -> Option<Vec<PlayerInfo>> {
        self.pending.remove(game_id)
    }
    pub fn contains(&self, game_id: &Uuid) -> bool {
        self.pending.contains_key(game_id)
    }
}

/// Manages all game sessions and pending games.
pub struct GameSessionManager {
    sessions: HashMap<Uuid, Addr<GameSession>>,
    pending_games: HashMap<Uuid, Vec<PlayerInfo>>,
}

impl GameSessionManager {
    /// Create a new manager.
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            pending_games: HashMap::new(),
        }
    }

    /// Register a pending game (called by matchmaking).
    pub fn register_pending_game(&mut self, game_id: Uuid, players: Vec<PlayerInfo>) {
        self.pending_games.insert(game_id, players);
    }

    /// Ensure a GameSession exists for the given game_id, creating it if needed.
    pub fn ensure_game_session(&mut self, game_id: Uuid) -> Result<Addr<GameSession>, String> {
        if let Some(addr) = self.sessions.get(&game_id) {
            // Session already exists, return it.
            return Ok(addr.clone());
        }
        // If not, check for pending players and create a new session.
        let players = self.pending_games.remove(&game_id)
            .ok_or_else(|| "No player group found for this game_id".to_string())?;
        let session = GameSession::new(game_id, players).start();
        self.sessions.insert(game_id, session.clone());
        Ok(session)
    }
}

impl Actor for GameSessionManager {
    type Context = Context<Self>;
}

impl Handler<RegisterPendingGame> for GameSessionManager {
    type Result = ();

    fn handle(&mut self, msg: RegisterPendingGame, _: &mut Context<Self>) -> Self::Result {
        self.register_pending_game(msg.game_id, msg.players);
    }
}

impl Handler<EnsureGameSession> for GameSessionManager {
    type Result = Result<Addr<GameSession>, String>;

    fn handle(&mut self, msg: EnsureGameSession, _: &mut Context<Self>) -> Self::Result {
        self.ensure_game_session(msg.game_id)
    }
}

/// Represents a running game session (one per game_id).
pub struct GameSession {
    pub game_id: Uuid,
    pub player_infos: Vec<PlayerInfo>,
    pub players: HashMap<WalletAddress, Addr<GameSessionActor>>,
    pub spectators: HashMap<WalletAddress, Addr<GameSessionActor>>,
    pub game_state: Option<GameState>,

    // Pre-game phase
    phase: GamePhase,
    mode_votes: HashMap<WalletAddress, GameMode>,
    mode_choice_deadline: Instant,
    mode_choice_timer: Option<SpawnHandle>,
    chosen_mode: Option<GameMode>,
    chosen_by: Option<WalletAddress>,

    // In-game phase
    pending_actions: HashMap<WalletAddress, PlayerAction>,
    turn_timer: Option<SpawnHandle>,
    turn_in_progress: bool,
}

/// Represents the current phase of the game.
#[derive(Debug, Clone, PartialEq)]
enum GamePhase {
    WaitingForModeChoice,
    InGame,
}

impl GameSession {
    /// Create a new game session for the given players.
    pub fn new(game_id: Uuid, player_infos: Vec<PlayerInfo>) -> Self {
        Self {
            game_id,
            player_infos,
            players: HashMap::new(),
            spectators: HashMap::new(),
            game_state: None,
            phase: GamePhase::WaitingForModeChoice,
            mode_votes: HashMap::new(),
            mode_choice_deadline: Instant::now() + Duration::from_secs(MODE_CHOICE_DURATION),
            mode_choice_timer: None,
            chosen_mode: None,
            chosen_by: None,
            pending_actions: HashMap::new(),
            turn_timer: None,
            turn_in_progress: false,
        }
    }

    /// Register a player or spectator session.
    fn handle_register_session(&mut self, msg: RegisterSession, _: &mut Context<Self>) {
        if msg.is_player {
            self.players.insert(msg.wallet.clone(), msg.addr.clone());
        } else {
            self.spectators.insert(msg.wallet.clone(), msg.addr.clone());
        }
        // Send the appropriate state depending on the phase.
        match self.phase {
            GamePhase::WaitingForModeChoice => {
                let now = Instant::now();
                let deadline_secs = self.mode_choice_deadline.saturating_duration_since(now).as_secs();
                let pre_game_msg = GamePreGameData {
                    modes: vec![GameMode::Classic, GameMode::Cracked],
                    deadline_secs,
                    players: self.player_infos.clone(),
                    grid_row: GRID_ROW,
                    grid_col: GRID_COL,
                };
                msg.addr.do_send(pre_game_msg);
            }
            GamePhase::InGame => {
                if let Some(ref state) = self.game_state {
                    msg.addr.do_send(GameStateUpdate { state: state.clone(), turn_duration: TURN_DURATION });
                }
            }
        }
    }

    /// Broadcast pre-game data to all players and spectators.
    fn broadcast_to_players_pre_game_data(&mut self, ctx: &mut Context<Self>) {
        let deadline_secs = self.mode_choice_deadline.saturating_duration_since(Instant::now()).as_secs();
        let msg = GamePreGameData {
            modes: vec![GameMode::Classic, GameMode::Cracked],
            deadline_secs,
            players: self.player_infos.clone(),
            grid_row: GRID_ROW,
            grid_col: GRID_COL,
        };
        for addr in self.players.values().chain(self.spectators.values()) {
            addr.do_send(msg.clone());
        }
        // If no timer is set, start one for the mode choice deadline.
        if self.mode_choice_timer.is_none() {
            let handle = ctx.run_later(Duration::from_secs(deadline_secs), |act, ctx| {
                act.finalize_mode_choice(ctx);
            });
            self.mode_choice_timer = Some(handle);
        }
    }

    /// Finalize the mode choice, either by votes or randomly if no votes.
    fn finalize_mode_choice(&mut self, ctx: &mut Context<Self>) {
        let (chosen_mode, chosen_by) = if !self.mode_votes.is_empty() {
            // If there are votes, pick one at random among the voters.
            let mut rng = rand::rng();
            let (chosen_player, mode) = self.mode_votes.iter().choose(&mut rng).unwrap();
            (mode.clone(), chosen_player.clone())
        } else {
            // If no votes, pick a mode and a player at random.
            let modes = [GameMode::Classic, GameMode::Cracked];
            let mut rng = rand::rng();
            let mode = *modes.iter().choose(&mut rng).unwrap();
            let chosen_player = self.player_infos.iter().choose(&mut rng).unwrap().id.clone();
            (mode, chosen_player)
        };
        self.chosen_mode = Some(chosen_mode.clone());
        self.chosen_by = Some(chosen_by.clone());
        // Notify all clients of the chosen mode.
        for addr in self.players.values().chain(self.spectators.values()) {
            addr.do_send(GameModeChosen {
                mode: self.chosen_mode.clone().expect("Mode should be chosen before broadcasting"),
                chosen_by: self.chosen_by.clone().expect("Chosen_by should be set before broadcasting"),
            });
        }

        // Initialize the game state with the chosen mode.
        self.game_state = Some(GameState::new(
            GRID_ROW, GRID_COL, self.player_infos.clone(), chosen_mode.clone(),
        ));
        self.phase = GamePhase::InGame;

        // Cancel the mode choice timer if it was set.
        if let Some(handle) = self.mode_choice_timer.take() {
            ctx.cancel_future(handle);
        }

        self.start_new_turn(ctx);
    }

    /// Register a mode vote from a player.
    fn receive_mode_vote(&mut self, player_id: WalletAddress, mode: GameMode, ctx: &mut Context<Self>) {
        // Only accept votes during the mode choice phase.
        if self.phase != GamePhase::WaitingForModeChoice {
            return;
        }
        self.mode_votes.insert(player_id, mode);

        // If all players have voted, finalize immediately.
        if self.mode_votes.len() >= self.player_infos.len() {
            self.finalize_mode_choice(ctx);
        }
    }

    /// Broadcast the current game state to all players and spectators.
    fn broadcast_to_players_game_state_update(&self, state: &GameState) {
        for addr in self.players.values().chain(self.spectators.values()) {
            addr.do_send(GameStateUpdate { state: state.clone(), turn_duration: TURN_DURATION });
        }
    }

    /// Broadcast the chosen mode to all clients.
    fn broadcast_to_players_mode_chosen(&self) {
        let mode = self.chosen_mode.clone().expect("Mode should be chosen before broadcasting");
        let chosen_by = self.chosen_by.clone().expect("Chosen_by should be set before broadcasting");
        for addr in self.players.values().chain(self.spectators.values()) {
            addr.do_send(GameModeChosen {
                mode,
                chosen_by: chosen_by.clone(),
            });
        }
    }

    /// Start the mode choice phase (used for restarts or new games).
    fn start_mode_choice(&mut self, ctx: &mut Context<Self>) {
        self.phase = GamePhase::WaitingForModeChoice;
        self.mode_choice_deadline = Instant::now() + Duration::from_secs(MODE_CHOICE_DURATION);
        self.mode_votes.clear();
        self.chosen_mode = None;

        self.broadcast_to_players_pre_game_data(ctx);

        info!("[GameSession] Mode choice started for game_id={}", self.game_id);
    }

    /// Send the current game state to all clients.
    pub fn send_state(&self) {
        if let Some(ref state) = self.game_state {
            debug!(
                "[GameSession] Broadcast GameState: game_id={} turn={} players={:?}",
                self.game_id,
                state.turn,
                state.players.iter().map(|p| &p.id).collect::<Vec<_>>()
            );
            self.broadcast_to_players_game_state_update(state);
        }
    }

    /// Start a new turn, resetting actions and launching the timer.
    fn start_new_turn(&mut self, ctx: &mut Context<Self>) {
        if self.game_state.is_none() {
            return;
        }
        self.turn_in_progress = true;
        self.pending_actions.clear();

        // Start the turn timer.
        let handle = ctx.run_later(Duration::from_secs(TURN_DURATION), |act, ctx| {
            act.resolve_turn(ctx);
        });
        self.turn_timer = Some(handle);

        // Optionally broadcast the new turn state.
        if let Some(ref state) = self.game_state {
            self.broadcast_to_players_game_state_update(state);
        }
    }

    /// Resolve the current turn, applying all actions and updating the game state.
    fn resolve_turn(&mut self, ctx: &mut Context<Self>) {
        if self.game_state.is_none() {
            return;
        }
        // Prevent double resolution of the same turn.
        if !self.turn_in_progress {
            return;
        }
        self.turn_in_progress = false;

        let state = self.game_state.as_mut().unwrap();

        // For each living player, if no action was received, default to Stay.
        for info in &self.player_infos {
            if !self.pending_actions.contains_key(&info.id) {
                if let Some(_idx) = state.players.iter().position(|p| p.username == info.username && p.is_alive) {
                    self.pending_actions.insert(info.id.clone(), PlayerAction::Move(Direction::Stay));
                }
            }
        }

        // Apply all actions in player order.
        for (i, info) in self.player_infos.iter().enumerate() {
            // Skip dead players.
            if let Some(player) = state.players.get(i) {
                if !player.is_alive { continue; }
            }
            if let Some(action) = self.pending_actions.get(&info.id) {
                state.apply_player_action(action.clone(), i);
            }
        }

        // Advance the turn counter.
        state.next_turn();

        // If more than one player is alive, start the next turn.
        if state.players.iter().filter(|p| p.is_alive).count() > 1 {
            self.start_new_turn(ctx);
        } else {
            // Game is over, notify all clients.
            let _winner = state.players.iter().find(|p| p.is_alive).map(|p| p.username.clone()).unwrap_or("No winner".to_string());
            for addr in self.players.values().chain(self.spectators.values()) {
                addr.do_send(GameStateUpdate { state: state.clone(), turn_duration: TURN_DURATION });
                // TODO: send a GameEnded message if needed
            }
        }
    }
}

impl Actor for GameSession {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // On actor start, broadcast pre-game data if in mode choice phase.
        if self.phase == GamePhase::WaitingForModeChoice {
            self.broadcast_to_players_pre_game_data(ctx);
        }
    }
}

/// Message to get a GameSession by game_id.
#[derive(Message)]
#[rtype(result = "Result<Addr<GameSession>, String>")]
pub struct GetGameSession {
    pub game_id: Uuid,
}

// Handler for GameModeVote: notifies all players of the vote and finalizes if all have voted.
impl Handler<GameModeVote> for GameSession {
    type Result = ();

    fn handle(&mut self, msg: GameModeVote, ctx: &mut Context<Self>) -> Self::Result {
        self.mode_votes.insert(msg.player_id.clone(), msg.mode.clone());
        let vote_update = GameModeVoteUpdate {
            player_id: msg.player_id.clone(),
            mode: msg.mode.clone(),
        };
        for addr in self.players.values().chain(self.spectators.values()) {
            addr.do_send(vote_update.clone());
        }
        // If all players have voted, finalize the mode choice.
        if self.mode_votes.len() >= self.player_infos.len() {
            self.finalize_mode_choice(ctx);
        }
    }
}

impl Handler<GetGameSession> for GameSessionManager {
    type Result = Result<Addr<GameSession>, String>;

    fn handle(&mut self, msg: GetGameSession, _: &mut Context<Self>) -> Self::Result {
        self.sessions.get(&msg.game_id)
            .cloned()
            .ok_or_else(|| "Game session not found".to_string())
    }
}

/// Message to check if a player is in a game.
#[derive(Message)]
#[rtype(result = "Result<bool, String>")]
pub struct IsPlayerInGame {
    pub game_id: Uuid,
    pub player_id: WalletAddress,
}

impl Handler<IsPlayerInGame> for GameSessionManager {
    type Result = Result<bool, String>;

    fn handle(&mut self, msg: IsPlayerInGame, _: &mut Context<Self>) -> Self::Result {
        self.sessions.get(&msg.game_id)
            .map(|session_addr| {
                session_addr.try_send(IsPlayer(msg.player_id.clone()))
                    .map(|_| true)
                    .unwrap_or(false)
            })
            .ok_or_else(|| "Game session not found".to_string())
    }
}

/// Message to check if a wallet is a player in the session.
#[derive(Message)]
#[rtype(result = "bool")]
pub struct IsPlayer(pub WalletAddress);

impl Handler<IsPlayer> for GameSession {
    type Result = bool;

    fn handle(&mut self, msg: IsPlayer, _: &mut Context<Self>) -> Self::Result {
        self.player_infos.iter().any(|p| p.id == msg.0)
    }
}

impl Handler<ProcessClientMessage> for GameSession {
    type Result = ();

    fn handle(&mut self, msg: ProcessClientMessage, ctx: &mut Context<Self>) -> Self::Result {
        // Ignore actions if the game hasn't started.
        if self.game_state.is_none() {
            return;
        }
        // Ignore actions if the turn is not in progress.
        if !self.turn_in_progress {
            warn!("[GameSession] Action received while turn is not in progress");
            return;
        }

        // Find the player's index.
        let player_index = match self.player_infos.iter().position(|p| p.id == msg.player_id) {
            Some(idx) => idx,
            None => return, // Unknown player
        };

        // Only allow actions from living players.
        if !self.game_state.as_ref().unwrap().players.get(player_index).map(|p| p.is_alive).unwrap_or(false) {
            warn!("[GameSession] Dead or unknown player tried to act: {}", msg.player_id);
            return;
        }

        // Prevent multiple actions per turn.
        if self.pending_actions.contains_key(&msg.player_id) {
            warn!("[GameSession] Player {} spamming, action already received this turn", msg.player_id);
            return;
        }

        // Register the action.
        self.pending_actions.insert(msg.player_id.clone(), msg.msg);

        // If all living players have acted, resolve the turn immediately.
        let alive_count = self.game_state.as_ref().unwrap().players.iter().filter(|p| p.is_alive).count();
        if self.pending_actions.len() >= alive_count {
            // Cancel the timer and resolve the turn.
            if let Some(handle) = self.turn_timer.take() {
                ctx.cancel_future(handle);
            }
            self.resolve_turn(ctx);
        }
    }
}

/// Message to register a session (player or spectator).
#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterSession {
    pub wallet: WalletAddress,
    pub addr: Addr<GameSessionActor>,
    pub is_player: bool,
}

/// Message to unregister a session (player or spectator).
#[derive(Message)]
#[rtype(result = "()")]
pub struct UnregisterSession {
    pub wallet: WalletAddress,
    pub is_player: bool,
}

impl Handler<RegisterSession> for GameSession {
    type Result = ();

    fn handle(&mut self, msg: RegisterSession, _: &mut Context<Self>) -> Self::Result {
        if msg.is_player {
            self.players.insert(msg.wallet.clone(), msg.addr.clone());
        } else {
            self.spectators.insert(msg.wallet.clone(), msg.addr.clone());
        }

        match self.phase {
            GamePhase::WaitingForModeChoice => {
                let now = Instant::now();
                let deadline_secs = self.mode_choice_deadline.saturating_duration_since(now).as_secs();
                let pre_game_msg = GamePreGameData {
                    modes: vec![GameMode::Classic, GameMode::Cracked],
                    deadline_secs,
                    players: self.player_infos.clone(),
                    grid_row: GRID_ROW,
                    grid_col: GRID_COL,
                };
                msg.addr.do_send(pre_game_msg);
            }
            GamePhase::InGame => {
                if let Some(ref state) = self.game_state {
                    msg.addr.do_send(GameStateUpdate { state: state.clone(), turn_duration: TURN_DURATION });
                }
            }
        }
    }
}

impl Handler<UnregisterSession> for GameSession {
    type Result = ();

    fn handle(&mut self, msg: UnregisterSession, _: &mut Context<Self>) -> Self::Result {
        if msg.is_player {
            self.players.remove(&msg.wallet);
        } else {
            self.spectators.remove(&msg.wallet);
        }
    }
}