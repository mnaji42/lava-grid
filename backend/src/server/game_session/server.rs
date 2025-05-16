use actix::prelude::*;
use std::collections::HashMap;
use actix::MessageResult;
use uuid::Uuid;
use log::{debug, warn};
use std::time::Duration;

use crate::game::state::GameState;
use crate::server::matchmaking::types::{PlayerInfo, WalletAddress};
use crate::server::game_session::session::GameSessionActor;
use crate::server::game_session::messages::{GameStateUpdate, ProcessClientMessage, ClientAction};
use crate::config::game::TURN_DURATION;
use crate::game::types::{Direction, GameMode};

pub struct GameSession {
    pub game_id: Uuid,
    pub player_infos: Vec<PlayerInfo>,
    pub players: HashMap<WalletAddress, Addr<GameSessionActor>>,
    pub spectators: HashMap<WalletAddress, Addr<GameSessionActor>>,
    pub game_state: GameState,

    pending_actions: HashMap<WalletAddress, ClientAction>,
    turn_timer: Option<SpawnHandle>,
    turn_in_progress: bool,
}

impl Actor for GameSession {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_new_turn(ctx);
    }
}

pub struct GameSessionManager {
    sessions: HashMap<Uuid, Addr<GameSession>>,
}

#[derive(Message)]
#[rtype(result = "Uuid")]
pub struct CreateGame {
    pub players: Vec<PlayerInfo>,
    pub mode: GameMode,
}

impl GameSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn create_game(&mut self, players: Vec<PlayerInfo>, mode: GameMode) -> Uuid {
        let game_id = Uuid::new_v4();
        let game_state = GameState::new(5, 5, players.clone(), mode);

        let session = GameSession {
            game_id,
            player_infos: players.clone(),
            players: HashMap::new(),
            spectators: HashMap::new(),
            game_state,
            pending_actions: HashMap::new(),
            turn_timer: None,
            turn_in_progress: false,
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
        MessageResult(self.create_game(msg.players, msg.mode))
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
        debug!(
            "[GameSession] Broadcast GameState: game_id={} turn={} players={:?}",
            self.game_id,
            self.game_state.turn,
            self.game_state.players.iter().map(|p| &p.id).collect::<Vec<_>>()
        );
        let state = self.game_state.clone();
        for addr in self.players.values().chain(self.spectators.values()) {
            addr.do_send(GameStateUpdate { state: state.clone(), turn_duration: TURN_DURATION });
        }
    }

    fn start_new_turn(&mut self, ctx: &mut Context<Self>) {
        self.turn_in_progress = true;
        self.pending_actions.clear();

        // Lance le timer de 5 secondes
        let handle = ctx.run_later(Duration::from_secs(TURN_DURATION), |act, ctx| {
            act.resolve_turn(ctx);
        });
        self.turn_timer = Some(handle);

        // Optionnel: broadcast un message "nouveau tour" si besoin
        self.send_state();
    }

    
    fn resolve_turn(&mut self, ctx: &mut Context<Self>) {
        // Garde anti-double appel
        if !self.turn_in_progress {
            // Déjà résolu ce tour, on ignore
            return;
        }
        self.turn_in_progress = false;

        // Pour chaque joueur vivant, si pas d'action, on met Stay
        for info in &self.player_infos {
            if !self.pending_actions.contains_key(&info.id) {
                // Trouver l'index du joueur dans game_state.players
                if let Some(idx) = self.game_state.players.iter().position(|p| p.username == info.username && p.is_alive) {
                    self.pending_actions.insert(info.id.clone(), ClientAction::Move(Direction::Stay));
                }
            }
        }

        // Appliquer toutes les actions dans l'ordre des player_infos
        for (i, info) in self.player_infos.iter().enumerate() {
            // Si le joueur est mort, on ne fait rien
            if let Some(player) = self.game_state.players.get(i) {
                if !player.is_alive { continue; }
            }
            if let Some(action) = self.pending_actions.get(&info.id) {
                self.game_state.apply_player_action(action.clone(), i);
            }
        }

        // Appliquer les règles globales à la fin du tour
        // self.game_state.apply_all_rules();

        // Incrémenter le tour UNE SEULE FOIS ici
        self.game_state.next_turn();

        // Préparer le prochain tour si la partie n'est pas finie
        if self.game_state.players.iter().filter(|p| p.is_alive).count() > 1 {
            self.start_new_turn(ctx);
        } else {
            // Partie terminée
            let winner = self.game_state.players.iter().find(|p| p.is_alive).map(|p| p.username.clone()).unwrap_or("No winner".to_string());
            for addr in self.players.values().chain(self.spectators.values()) {
                addr.do_send(GameStateUpdate { state: self.game_state.clone(), turn_duration: TURN_DURATION });
                // TODO: envoyer un message GameEnded si besoin
            }
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
        if !self.game_state.players.get(player_index).map(|p| p.is_alive).unwrap_or(false) {
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
        let alive_count = self.game_state.players.iter().filter(|p| p.is_alive).count();
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
        let state = self.game_state.clone();
        msg.addr.do_send(GameStateUpdate { state, turn_duration: TURN_DURATION });
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