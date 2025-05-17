use actix::prelude::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;
use log::{info, warn, debug};

use crate::game::state::GameState;
use crate::server::matchmaking::types::{PlayerInfo, WalletAddress};
use crate::server::game_session::session::{GameSessionActor};
use crate::config::game::{TURN_DURATION, MODE_CHOICE_DURATION, GRID_ROW, GRID_COL};
use crate::game::types::{GameMode, Direction};
use crate::server::game_session::messages::{
    GameStateUpdate, ProcessClientMessage, PlayerAction, RegisterPendingGame, EnsureGameSession,
    GameModeChosen, GamePreGameData, GameModeVote, GameModeVoteUpdate
};
use rand::prelude::IteratorRandom;

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

pub struct GameSessionManager {
    sessions: HashMap<Uuid, Addr<GameSession>>,
    pending_games: HashMap<Uuid, Vec<PlayerInfo>>,
}

impl GameSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            pending_games: HashMap::new(),
        }
    }

    pub fn register_pending_game(&mut self, game_id: Uuid, players: Vec<PlayerInfo>) {
        self.pending_games.insert(game_id, players);
    }

    pub fn ensure_game_session(&mut self, game_id: Uuid) -> Result<Addr<GameSession>, String> {
        if let Some(addr) = self.sessions.get(&game_id) {
            return Ok(addr.clone());
        }
        let players = self.pending_games.remove(&game_id)
            .ok_or_else(|| "Aucun groupe de joueurs trouvé pour ce game_id".to_string())?;
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

pub struct GameSession {
    pub game_id: Uuid,
    pub player_infos: Vec<PlayerInfo>,
    pub players: HashMap<WalletAddress, Addr<GameSessionActor>>,
    pub spectators: HashMap<WalletAddress, Addr<GameSessionActor>>,
    pub game_state: Option<GameState>,

    // Pré-game
    phase: GamePhase,
    mode_votes: HashMap<WalletAddress, GameMode>,
    mode_choice_deadline: Instant,
    mode_choice_timer: Option<SpawnHandle>,
    chosen_mode: Option<GameMode>,
    chosen_by: Option<WalletAddress>,

    // In-game
    pending_actions: HashMap<WalletAddress, PlayerAction>,
    turn_timer: Option<SpawnHandle>,
    turn_in_progress: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum GamePhase {
    WaitingForModeChoice,
    InGame,
}

impl GameSession {
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

    fn handle_register_session(&mut self, msg: RegisterSession, _: &mut Context<Self>) {
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
        // Timer pour la deadline si besoin...
        if self.mode_choice_timer.is_none() {
            let handle = ctx.run_later(Duration::from_secs(deadline_secs), |act, ctx| {
                act.finalize_mode_choice(ctx);
            });
            self.mode_choice_timer = Some(handle);
        }
    }

    fn finalize_mode_choice(&mut self, ctx: &mut Context<Self>) {
        let (chosen_mode, chosen_by) = if !self.mode_votes.is_empty() {
            let mut rng = rand::rng();
            let (chosen_player, mode) = self.mode_votes.iter().choose(&mut rng).unwrap();
            (mode.clone(), chosen_player.clone())
        } else {
            let modes = [GameMode::Classic, GameMode::Cracked];
            let mut rng = rand::rng();
            let mode = *modes.iter().choose(&mut rng).unwrap();
            let chosen_player = self.player_infos.iter().choose(&mut rng).unwrap().id.clone();
            (mode, chosen_player)
        };
        self.chosen_mode = Some(chosen_mode.clone());
        self.chosen_by = Some(chosen_by.clone());
        for addr in self.players.values().chain(self.spectators.values()) {
            addr.do_send(GameModeChosen {
                mode: self.chosen_mode.clone().expect("Mode should be chosen before broadcasting"),
                chosen_by: self.chosen_by.clone().expect("Chosen_by should be set before broadcasting"),
            });
        }

        // Initialiser la partie
        self.game_state = Some(GameState::new(
            GRID_ROW, GRID_COL, self.player_infos.clone(), chosen_mode.clone(),
        ));
        self.phase = GamePhase::InGame;

        if let Some(handle) = self.mode_choice_timer.take() {
            ctx.cancel_future(handle);
        }

        self.start_new_turn(ctx);
    }

    fn receive_mode_vote(&mut self, player_id: WalletAddress, mode: GameMode, ctx: &mut Context<Self>) {
        if self.phase != GamePhase::WaitingForModeChoice {
            return;
        }
        self.mode_votes.insert(player_id, mode);

        // Si tous les joueurs ont voté, on peut avancer
        if self.mode_votes.len() >= self.player_infos.len() {
            self.finalize_mode_choice(ctx);
        }
    }

    fn broadcast_to_players_game_state_update(&self, state: &GameState) {
        for addr in self.players.values().chain(self.spectators.values()) {
            addr.do_send(GameStateUpdate { state: state.clone(), turn_duration: TURN_DURATION });
        }
    }

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


    fn start_mode_choice(&mut self, ctx: &mut Context<Self>) {
        self.phase = GamePhase::WaitingForModeChoice;
        self.mode_choice_deadline = Instant::now() + Duration::from_secs(MODE_CHOICE_DURATION);
        self.mode_votes.clear();
        self.chosen_mode = None;

        // Correction : appel sans arguments supplémentaires
        self.broadcast_to_players_pre_game_data(ctx);

        // Le timer est déjà géré dans broadcast_to_players_pre_game_data
        info!("[GameSession] Mode choice started for game_id={}", self.game_id);
    }

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

    fn start_new_turn(&mut self, ctx: &mut Context<Self>) {
        if self.game_state.is_none() {
            return;
        }
        self.turn_in_progress = true;
        self.pending_actions.clear();

        // Lance le timer de 5 secondes
        let handle = ctx.run_later(Duration::from_secs(TURN_DURATION), |act, ctx| {
            act.resolve_turn(ctx);
        });
        self.turn_timer = Some(handle);

        // Optionnel: broadcast un message "nouveau tour" si besoin
        if let Some(ref state) = self.game_state {
            self.broadcast_to_players_game_state_update(state);
        }
    }

    fn resolve_turn(&mut self, ctx: &mut Context<Self>) {
        if self.game_state.is_none() {
            return;
        }
        // Garde anti-double appel
        if !self.turn_in_progress {
            // Déjà résolu ce tour, on ignore
            return;
        }
        self.turn_in_progress = false;

        let state = self.game_state.as_mut().unwrap();

        // Pour chaque joueur vivant, si pas d'action, on met Stay
        for info in &self.player_infos {
            if !self.pending_actions.contains_key(&info.id) {
                if let Some(_idx) = state.players.iter().position(|p| p.username == info.username && p.is_alive) {
                    self.pending_actions.insert(info.id.clone(), PlayerAction::Move(Direction::Stay));
                }
            }
        }

        // Appliquer toutes les actions dans l'ordre des player_infos
        for (i, info) in self.player_infos.iter().enumerate() {
            // Si le joueur est mort, on ne fait rien
            if let Some(player) = state.players.get(i) {
                if !player.is_alive { continue; }
            }
            if let Some(action) = self.pending_actions.get(&info.id) {
                state.apply_player_action(action.clone(), i);
            }
        }

        // Incrémenter le tour UNE SEULE FOIS ici
        state.next_turn();

        // Préparer le prochain tour si la partie n'est pas finie
        if state.players.iter().filter(|p| p.is_alive).count() > 1 {
            self.start_new_turn(ctx);
        } else {
            // Partie terminée
            let _winner = state.players.iter().find(|p| p.is_alive).map(|p| p.username.clone()).unwrap_or("No winner".to_string());
            for addr in self.players.values().chain(self.spectators.values()) {
                addr.do_send(GameStateUpdate { state: state.clone(), turn_duration: TURN_DURATION });
                // TODO: envoyer un message GameEnded si besoin
            }
        }
    }
}

impl Actor for GameSession {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        if self.phase == GamePhase::WaitingForModeChoice {
            self.broadcast_to_players_pre_game_data(ctx);
        }
    }
}

#[derive(Message)]
#[rtype(result = "Result<Addr<GameSession>, String>")]
pub struct GetGameSession {
    pub game_id: Uuid,
}

// Handler pour GameModeVote (notifie tous les joueurs du vote reçu)
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
        if self.game_state.is_none() {
            return;
        }
        // Si le tour n'est pas en cours, on ignore
        if !self.turn_in_progress {
            warn!("[GameSession] Action reçue alors que le tour n'est pas en cours");
            return;
        }

        // Trouver l'index du joueur dans self.player_infos
        let player_index = match self.player_infos.iter().position(|p| p.id == msg.player_id) {
            Some(idx) => idx,
            None => return, // joueur inconnu
        };

        // Vérifier que le joueur est vivant
        if !self.game_state.as_ref().unwrap().players.get(player_index).map(|p| p.is_alive).unwrap_or(false) {
            warn!("[GameSession] Joueur mort ou inexistant tente d'agir: {}", msg.player_id);
            return;
        }

        // Anti-spam: une seule action par tour
        if self.pending_actions.contains_key(&msg.player_id) {
            warn!("[GameSession] Joueur {} spamme, action déjà reçue ce tour", msg.player_id);
            return;
        }

        // Enregistrer l'action
        self.pending_actions.insert(msg.player_id.clone(), msg.msg);

        // Si toutes les actions sont reçues, on résout le tour immédiatement
        let alive_count = self.game_state.as_ref().unwrap().players.iter().filter(|p| p.is_alive).count();
        if self.pending_actions.len() >= alive_count {
            // Annule le timer
            if let Some(handle) = self.turn_timer.take() {
                ctx.cancel_future(handle);
            }
            self.resolve_turn(ctx);
        }
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