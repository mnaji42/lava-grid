// Logique Matchmaking (peut etre qu'on aura besoin de la recuperer)

use actix::prelude::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};
// use rand::seq::SliceRandom; // (optionnel, pour d'autres méthodes)
use rand::prelude::IndexedRandom; // <-- AJOUTE CETTE LIGNE
use uuid::Uuid;

use super::types::{PlayerInfo, WalletAddress};
use super::messages::{ServerWsMessage, MatchmakingState};
use super::session::MatchmakingSession;
use crate::config::matchmaking::{MIN_PLAYERS, COUNTDOWN_DURATION_SECS, MAX_PLAYERS, PRE_GAME_WARNING_TIME};
use crate::config::game::MODE_CHOICE_DURATION;
use crate::server::game_session::server::{GameSessionManager, CreateGame}; // Import correct
use crate::game::types::GameMode;

type SessionAddr = Addr<MatchmakingSession>;

pub struct MatchmakingServer {
    sessions: HashMap<WalletAddress, (PlayerInfo, SessionAddr)>,
    countdown: Option<CountdownHandles>,
    game_session_manager: Addr<GameSessionManager>,
    pending_game: Option<PendingGame>,
}

struct CountdownHandles {
    warning_handle: SpawnHandle,
    game_start_handle: SpawnHandle,
    start_time: Instant,
}

// SUPPRIMÉ: #[derive(Message)] pub struct CreateGame {...}

struct PendingGame {
    players: Vec<PlayerInfo>,
    mode_votes: HashMap<WalletAddress, GameMode>,
    mode_choice_deadline: Instant,
    sessions: HashMap<WalletAddress, SessionAddr>,
}

impl MatchmakingServer {
    pub fn new(game_session_manager: Addr<GameSessionManager>) -> Self {
        Self {
            sessions: HashMap::new(),
            countdown: None,
            game_session_manager,
            pending_game: None,
        }
    }

    fn broadcast(&self, msg: ServerWsMessage) {
        if let Some(pending) = &self.pending_game {
            for (_, addr) in &pending.sessions {
                addr.do_send(msg.clone());
            }
        }
        for (_, (_player_info, addr)) in &self.sessions {
            addr.do_send(msg.clone());
        }
    }

    fn get_state(&self) -> MatchmakingState {
        let time_remaining = self.countdown.as_ref().map_or(COUNTDOWN_DURATION_SECS, |h| {
            COUNTDOWN_DURATION_SECS.saturating_sub(h.start_time.elapsed().as_secs())
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
            Duration::from_secs(COUNTDOWN_DURATION_SECS - PRE_GAME_WARNING_TIME), 
            |act, _| {
                act.broadcast(ServerWsMessage::update_state(act.get_state()));
            }
        );

        let game_start_handle = ctx.run_later(
            Duration::from_secs(COUNTDOWN_DURATION_SECS),
            |act, ctx| act.start_mode_choice(ctx),
        );

        self.countdown = Some(CountdownHandles {
            warning_handle,
            game_start_handle,
            start_time: Instant::now(),
        });
    }

    fn start_mode_choice(&mut self, ctx: &mut Context<Self>) {
        // Figer la liste des joueurs et sessions
        let players: Vec<PlayerInfo> = self.sessions.values().map(|(info, _)| info.clone()).collect();
        let sessions: HashMap<WalletAddress, SessionAddr> = self.sessions.iter().map(|(id, (_info, addr))| (id.clone(), addr.clone())).collect();
        self.pending_game = Some(PendingGame {
            players: players.clone(),
            mode_votes: HashMap::new(),
            mode_choice_deadline: Instant::now() + Duration::from_secs(MODE_CHOICE_DURATION),
            sessions,
        });
        self.sessions.clear();
        // Envoyer la liste des modes disponibles à tous les joueurs
        self.broadcast(ServerWsMessage::AvailableGameModes {
            modes: vec![GameMode::Classic, GameMode::Cracked],
        });
        // Démarrer un timer pour la deadline
        let _handle = ctx.run_later(Duration::from_secs(MODE_CHOICE_DURATION), |act, ctx| {
            act.finalize_mode_choice(ctx);
        });
        // Annuler le countdown si besoin
        if let Some(handles) = self.countdown.take() {
            ctx.cancel_future(handles.warning_handle);
            ctx.cancel_future(handles.game_start_handle);
        }
    }

    fn finalize_mode_choice(&mut self, ctx: &mut Context<Self>) {
        if let Some(pending) = self.pending_game.take() {
            let mut rng = rand::rng();
            let mode = if !pending.mode_votes.is_empty() {
                // Ici, on collecte les clés (WalletAddress) dans un Vec<&WalletAddress>
                let keys: Vec<&WalletAddress> = pending.mode_votes.keys().collect();
                let chosen_player = keys.choose(&mut rng).unwrap();
                pending.mode_votes[*chosen_player].clone()
            } else {
                // Aucun vote, choisir au hasard parmi les modes disponibles
                let all_modes = [GameMode::Classic, GameMode::Cracked];
                *all_modes.choose(&mut rng).unwrap()
            };
            // Créer la partie avec le mode choisi
            let players = pending.players.clone();
            let sessions = pending.sessions.clone();
            self.game_session_manager
                .send(CreateGame { players: players.clone(), mode })
                .into_actor(self)
                .then(move |res, _act, _ctx| {
                    match res {
                        Ok(game_id) => {
                            for (_id, addr) in sessions {
                                addr.do_send(ServerWsMessage::game_started(game_id));
                            }
                        }
                        Err(_e) => {
                            for (_id, addr) in sessions {
                                addr.do_send(ServerWsMessage::error("Erreur création partie"));
                            }
                        }
                    }
                    fut::ready(())
                })
                .wait(ctx);
        }
    }

    // Handler pour recevoir les votes des joueurs (à ajouter)
    // ...

    fn start_game(&mut self, _ctx: &mut Context<Self>) {
        // Ancienne logique, à ne plus utiliser
        // On ne doit plus appeler cette fonction directement
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
        // Si on est en mode choix du mode, refuser toute nouvelle connexion
        if self.pending_game.is_some() {
            // Optionnel: envoyer une erreur au client
            return;
        }
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
            self.start_mode_choice(ctx);
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
        // Si la partie est figée (pending_game), refuser la désinscription
        if self.pending_game.is_some() {
            // Optionnel: envoyer une erreur au client
            return;
        }
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