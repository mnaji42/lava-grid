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
use crate::config::game::{TURN_DURATION, GRID_ROW, GRID_COL};
use crate::server::session_utils::{is_game_session_addr_valid, is_game_session_spectator_addr_valid};
use crate::game::types::GameMode;
use crate::server::game_session::messages::{
    GameStateUpdate, ProcessClientMessage, PlayerAction, RegisterPendingGame, EnsureGameSession,
    GameModeVote, SessionKicked
};
use crate::server::game_session::mode_choice::ModeChoice;
use crate::server::game_session::turn_resolution::{start_new_turn, resolve_turn};

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

    // Mode choice phase
    pub mode_choice: ModeChoice,

    // In-game phase
    pub pending_actions: HashMap<WalletAddress, PlayerAction>,
    pub turn_timer: Option<SpawnHandle>,
    pub turn_in_progress: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum GamePhase {
    WaitingForModeChoice,
    InGame,
}

impl GameSession {
    /// Create a new game session for the given players.
    pub fn new(game_id: Uuid, player_infos: Vec<PlayerInfo>) -> Self {
        let required_players = player_infos.len();
        Self {
            game_id,
            player_infos,
            players: HashMap::new(),
            spectators: HashMap::new(),
            game_state: None,
            mode_choice: ModeChoice::new(required_players),
            pending_actions: HashMap::new(),
            turn_timer: None,
            turn_in_progress: false,
        }
    }
    

    /// Start the mode choice phase (used for restarts or new games).
    fn start_mode_choice(&mut self, ctx: &mut Context<Self>) {
        self.mode_choice.reset();
        self.mode_choice.broadcast_to_players_pre_game_data(
            &self.players,
            &self.spectators,
            &self.player_infos,
        );
        // Start the timer for mode choice deadline.
        let deadline_secs = self.mode_choice.deadline.saturating_duration_since(Instant::now()).as_secs();
        let handle = ctx.run_later(Duration::from_secs(deadline_secs), |act, ctx| {
            act.finalize_mode_choice(ctx);
        });
        self.mode_choice.timer = Some(handle);
        info!("[GameSession] Mode choice started for game_id={}", self.game_id);
    }

    /// Finalize the mode choice, either by votes or randomly if no votes.
    fn finalize_mode_choice(&mut self, ctx: &mut Context<Self>) {
        self.mode_choice.finalize_mode_choice(
            &self.player_infos,
            &self.players,
            &self.spectators,
        );
        // Initialize the game state with the chosen mode.
        let chosen_mode = self.mode_choice.chosen_mode.clone().expect("Mode should be chosen before broadcasting");
        self.game_state = Some(GameState::new(
            GRID_ROW, GRID_COL, self.player_infos.clone(), chosen_mode,
        ));
        // Cancel the mode choice timer if it was set.
        if let Some(handle) = self.mode_choice.timer.take() {
            ctx.cancel_future(handle);
        }
        self.turn_in_progress = false;
        start_new_turn(self, ctx);
    }

    /// Register a mode vote from a player.
    fn receive_mode_vote(&mut self, player_id: WalletAddress, mode: GameMode, ctx: &mut Context<Self>) {
        let all_voted = self.mode_choice.receive_mode_vote(
            player_id,
            mode,
            &self.players,
            &self.spectators,
        );
        if all_voted {
            self.finalize_mode_choice(ctx);
        }
    }

    /// Broadcast the current game state to all players and spectators.
    pub fn send_state(&self) {
        if let Some(ref state) = self.game_state {
            debug!(
                "[GameSession] Broadcast GameState: game_id={} turn={} players={:?}",
                self.game_id,
                state.turn,
                state.players.iter().map(|p| &p.id).collect::<Vec<_>>()
            );
            for addr in self.players.values().chain(self.spectators.values()) {
                addr.do_send(GameStateUpdate { state: state.clone(), turn_duration: TURN_DURATION });
            }
        }
    }
}

impl Actor for GameSession {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // On actor start, broadcast pre-game data if in mode choice phase.
        if self.game_state.is_none() {
            self.start_mode_choice(ctx);
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
        self.receive_mode_vote(msg.player_id, msg.mode, ctx);
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
        // Verify that the session address matches the one registered for this wallet
        if !is_game_session_addr_valid(&self.players, &msg.player_id, &msg.addr) {
            warn!("[GameSession] Action ignored: session addr mismatch for wallet={}", msg.player_id);
            return;
        }

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
            resolve_turn(self, ctx);
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
    pub addr: Addr<GameSessionActor>,
    pub is_player: bool,
}

impl Handler<RegisterSession> for GameSession {
    type Result = ();

    fn handle(&mut self, msg: RegisterSession, _: &mut Context<Self>) -> Self::Result {
        if msg.is_player {
            if let Some(old_addr) = self.players.get(&msg.wallet) {
                old_addr.do_send(SessionKicked {
                    reason: "Another session has connected with your wallet in this game.".to_string(),
                });
            }
            self.players.insert(msg.wallet.clone(), msg.addr.clone());
        } else {
            if let Some(old_addr) = self.spectators.get(&msg.wallet) {
                old_addr.do_send(SessionKicked {
                    reason: "Another session has connected with your wallet in this game (spectator).".to_string(),
                });
            }
            self.spectators.insert(msg.wallet.clone(), msg.addr.clone());
        }

        if self.game_state.is_none() {
            self.mode_choice.broadcast_to_players_pre_game_data(
                &self.players,
                &self.spectators,
                &self.player_infos,
            );
        } else {
            if let Some(ref state) = self.game_state {
                msg.addr.do_send(GameStateUpdate { state: state.clone(), turn_duration: TURN_DURATION });
            }
        }
    }
}

impl Handler<UnregisterSession> for GameSession {
    type Result = ();

    fn handle(&mut self, msg: UnregisterSession, _: &mut Context<Self>) -> Self::Result {
        if msg.is_player {
            // Only remove the player if the address matches the registered one
            if is_game_session_addr_valid(&self.players, &msg.wallet, &msg.addr) {
                self.players.remove(&msg.wallet);
            } else {
                warn!("[GameSession] Unregister ignored: session addr mismatch for wallet={}", msg.wallet);
            }
        } else {
            // For spectators, apply the same verification
            if is_game_session_spectator_addr_valid(&self.spectators, &msg.wallet, &msg.addr) {
                self.spectators.remove(&msg.wallet);
            } else {
                warn!("[GameSession] Spectator unregister ignored: session addr mismatch for wallet={}", msg.wallet);
            }
        }
    }
}
