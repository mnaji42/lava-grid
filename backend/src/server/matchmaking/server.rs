use actix::prelude::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use super::types::{PlayerInfo, WalletAddress};
use super::messages::{ServerWsMessage, MatchmakingState};
use super::session::MatchmakingSession;
use crate::config::matchmaking::{MIN_PLAYERS, MAX_PLAYERS, COUNTDOWN_DURATION_SECS};
use crate::server::game_session::server::{GameSessionManager, CreateGame};
use crate::game::types::GameMode;
use log::{info, warn, debug};

type SessionAddr = Addr<MatchmakingSession>;

#[derive(Debug, Clone)]
struct ConnectedPlayer {
    info: PlayerInfo,
    addr: SessionAddr,
}

struct CountdownHandle {
    handle: SpawnHandle,
    start_time: Instant,
}

pub struct MatchmakingServer {
    lobby_players: HashMap<WalletAddress, ConnectedPlayer>,
    ready_groups: Vec<HashMap<WalletAddress, ConnectedPlayer>>,
    countdown: Option<CountdownHandle>,
    game_session_manager: Addr<GameSessionManager>,
}

impl MatchmakingServer {
    pub fn new(game_session_manager: Addr<GameSessionManager>) -> Self {
        Self {
            lobby_players: HashMap::new(),
            ready_groups: Vec::new(),
            countdown: None,
            game_session_manager,
        }
    }

    fn broadcast(&self, msg: ServerWsMessage) {
        for player in self.lobby_players.values() {
            player.addr.do_send(msg.clone());
        }
        for group in &self.ready_groups {
            for player in group.values() {
                player.addr.do_send(msg.clone());
            }
        }
    }

    fn send_state(&self) {
        let state = self.get_state();
        self.broadcast(ServerWsMessage::UpdateState(state));
    }

    fn get_state(&self) -> MatchmakingState {
        let countdown_active = self.countdown.is_some();
        let countdown_remaining = self.countdown.as_ref().map(|c| {
            COUNTDOWN_DURATION_SECS.saturating_sub(c.start_time.elapsed().as_secs())
        });
        let ready_players: Vec<PlayerInfo> = self.ready_groups
            .iter()
            .flat_map(|group| group.values().map(|p| p.info.clone()))
            .collect();
        MatchmakingState {
            lobby_players: self.lobby_players.values().map(|p| p.info.clone()).collect(),
            ready_players,
            countdown_active,
            countdown_remaining,
        }
    }

    fn start_countdown(&mut self, ctx: &mut Context<Self>) {
        if self.countdown.is_some() {
            return;
        }
        self.send_state();
        let handle = ctx.run_later(Duration::from_secs(COUNTDOWN_DURATION_SECS), |act, ctx| {
            act.try_launch_next_game(ctx);
        });
        self.countdown = Some(CountdownHandle {
            handle,
            start_time: Instant::now(),
        });
        info!("[Matchmaking] Countdown started for next group");
    }

    fn cancel_countdown(&mut self, ctx: &mut Context<Self>) {
        if let Some(countdown) = self.countdown.take() {
            ctx.cancel_future(countdown.handle);
            self.send_state();
            info!("[Matchmaking] Countdown cancelled");
        }
    }

    // fn send_to_players(&self, player_ids: &[WalletAddress], msg: ServerWsMessage) {
    //     for id in player_ids {
    //         if let Some(player) = self.lobby_players.get(id) {
    //             player.addr.do_send(msg.clone());
    //         }
    //         if let Some(player) = self.ready_players.get(id) {
    //             player.addr.do_send(msg.clone());
    //         }
    //     }
    // }

    // fn broadcast_except(&self, excluded_ids: &[WalletAddress], msg: ServerWsMessage) {
    //     for (id, player) in self.lobby_players.iter() {
    //         if !excluded_ids.contains(id) {
    //             player.addr.do_send(msg.clone());
    //         }
    //     }
    //     for (id, player) in self.ready_players.iter() {
    //         if !excluded_ids.contains(id) {
    //             player.addr.do_send(msg.clone());
    //         }
    //     }
    // }

    fn try_launch_next_game(&mut self, ctx: &mut Context<Self>) {
        if let Some((group_idx, group)) = self.ready_groups.iter().enumerate().find(|(_, g)| g.len() >= MIN_PLAYERS) {
            let player_infos: Vec<PlayerInfo> = group.values().map(|p| p.info.clone()).collect();
            let player_addrs: Vec<SessionAddr> = group.values().map(|p| p.addr.clone()).collect();
            let group_ids: Vec<WalletAddress> = group.keys().cloned().collect();

            // On retire le countdown (la partie va se lancer)
            self.cancel_countdown(ctx);

            let game_mode = GameMode::Classic; // TODO: support voting/choice
            let game_session_manager = self.game_session_manager.clone();

            // On clone l’index pour le move dans la closure
            let group_idx_clone = group_idx;

            game_session_manager
                .send(CreateGame { players: player_infos.clone(), mode: game_mode })
                .into_actor(self)
                .then(move |res, act, ctx| {
                    match res {
                        Ok(game_id) => {
                            for addr in &player_addrs {
                                addr.do_send(ServerWsMessage::GameStarted { game_id });
                            }
                            info!("[Matchmaking] Game started with {} players", player_addrs.len());
                            // On supprime le groupe de la liste (clean)
                            if group_idx_clone < act.ready_groups.len() {
                                act.ready_groups.remove(group_idx_clone);
                            }
                            act.send_state();
                        }
                        Err(_e) => {
                            // TODO: Limiter le nombre de tentatives pour éviter les boucles infinies
                            warn!("[Matchmaking] Game creation failed for group, will retry");
                            // On peut décider de relancer la création ou de notifier les joueurs d’une erreur
                            // Ici, on ne supprime pas le groupe pour pouvoir retenter
                            act.broadcast(ServerWsMessage::Error {
                                message: "Erreur création partie".to_string(),
                            });
                        }
                    }
                    // Après la tentative, si un autre groupe est prêt, on relance le countdown si besoin
                    if let Some(first_group) = act.ready_groups.first() {
                        if first_group.len() >= MIN_PLAYERS && first_group.len() < MAX_PLAYERS && act.countdown.is_none() {
                            act.start_countdown(ctx);
                        }
                    }
                    fut::ready(())
                })
                .wait(ctx);
        }
    }

    fn add_or_update_lobby_player(&mut self, player_id: WalletAddress, addr: SessionAddr, username: String) {
        let player_info = PlayerInfo {
            id: player_id.clone(),
            username,
        };
        self.lobby_players.insert(player_id, ConnectedPlayer {
            info: player_info,
            addr,
        });
    }

    fn remove_player_from_ready_groups(&mut self, player_id: &WalletAddress) -> Option<ConnectedPlayer> {
        for group in &mut self.ready_groups {
            if let Some(player) = group.remove(player_id) {
                return Some(player);
            }
        }
        None
    }

    fn find_group_of_player_mut(&mut self, player_id: &WalletAddress) -> Option<&mut HashMap<WalletAddress, ConnectedPlayer>> {
        self.ready_groups.iter_mut().find(|g| g.contains_key(player_id))
    }

    fn refund_player(&self, player_id: &WalletAddress) {
        // TODO: Implémenter la logique de remboursement
        debug!("[Matchmaking] Refund requested for player {}", player_id);
    }
}



#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub player_id: WalletAddress,
    pub addr: SessionAddr,
    pub username: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Leave {
    pub player_id: WalletAddress,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Pay {
    pub player_id: WalletAddress,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CancelPayment {
    pub player_id: WalletAddress,
}

impl Actor for MatchmakingServer {
    type Context = Context<Self>;
}

impl Handler<Join> for MatchmakingServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _ctx: &mut Self::Context) -> Self::Result {
        // Si déjà dans un groupe prêt, update l’adresse (reconnexion)
        if let Some(group) = self.find_group_of_player_mut(&msg.player_id) {
            if let Some(player) = group.get_mut(&msg.player_id) {
                player.addr = msg.addr;
                debug!("[Matchmaking] Player {} reconnected in ready_groups", msg.player_id);
                self.send_state();
                return;
            }
        }
        // Si déjà dans le lobby, update l’adresse (reconnexion)
        if let Some(player) = self.lobby_players.get_mut(&msg.player_id) {
            player.addr = msg.addr;
            debug!("[Matchmaking] Player {} reconnected in lobby_players", msg.player_id);
            self.send_state();
            return;
        }
        // Sinon, nouveau joueur : ajout dans lobby_players
        self.add_or_update_lobby_player(msg.player_id.clone(), msg.addr, msg.username);
        debug!("[Matchmaking] Player {} joined lobby_players", msg.player_id);
        self.send_state();
    }
}

impl Handler<Leave> for MatchmakingServer {
    type Result = ();

    fn handle(&mut self, msg: Leave, ctx: &mut Self::Context) -> Self::Result {
        if self.lobby_players.remove(&msg.player_id).is_some() {
            debug!("[Matchmaking] Player {} left lobby_players", msg.player_id);
            self.send_state();
            return;
        }

        let countdown_active = self.countdown.is_some();
        if let Some(group) = self.find_group_of_player_mut(&msg.player_id) {
            if countdown_active {
                debug!("[Matchmaking] Player {} tried to leave during countdown (not allowed)", msg.player_id);
                // TODO: envoyer un message d’erreur au client
                return;
            }
            group.remove(&msg.player_id);
            debug!("[Matchmaking] Player {} left ready_groups (removed, not put back in lobby)", msg.player_id);
            self.ready_groups.retain(|g| !g.is_empty());
            self.refund_player(&msg.player_id);
            self.send_state();
            return;
        }
    }
}

impl Handler<Pay> for MatchmakingServer {
    type Result = ();

    fn handle(&mut self, msg: Pay, ctx: &mut Self::Context) -> Self::Result {
        if self.find_group_of_player_mut(&msg.player_id).is_some() {
            debug!("[Matchmaking] Player {} tried to pay but is already ready", msg.player_id);
            // TODO: envoyer un message d’erreur au client
            return;
        }
        let player = match self.lobby_players.remove(&msg.player_id) {
            Some(p) => p,
            None => {
                debug!("[Matchmaking] Player {} tried to pay but is not in lobby_players", msg.player_id);
                // TODO: envoyer un message d’erreur au client
                return;
            }
        };

        let mut added_to_group = false;
        for group in &mut self.ready_groups {
            if group.len() < MAX_PLAYERS {
                group.insert(msg.player_id.clone(), player.clone());
                added_to_group = true;
                break;
            }
        }
        if !added_to_group {
            let mut new_group = HashMap::new();
            new_group.insert(msg.player_id.clone(), player.clone());
            self.ready_groups.push(new_group);
        }

        debug!("[Matchmaking] Player {} moved to ready_groups", msg.player_id);

        if let Some(first_group) = self.ready_groups.first() {
            if first_group.len() >= MAX_PLAYERS {
                self.cancel_countdown(ctx);
                self.try_launch_next_game(ctx);
            } else if first_group.len() >= MIN_PLAYERS && self.countdown.is_none() {
                self.start_countdown(ctx);
            }
        }
        self.send_state();
    }
}

impl Handler<CancelPayment> for MatchmakingServer {
    type Result = ();

    fn handle(&mut self, msg: CancelPayment, _ctx: &mut Self::Context) -> Self::Result {
        let countdown_active = self.countdown.is_some();
        let group = self.find_group_of_player_mut(&msg.player_id);
        if group.is_none() {
            return;
        }
        let group = group.unwrap();

        if countdown_active {
            if let Some(player) = group.get(&msg.player_id) {
                player.addr.do_send(ServerWsMessage::Error {
                    message: "Cannot cancel payment: game is about to start.".to_string(),
                });
            }
            return;
        }

        if let Some(player) = group.remove(&msg.player_id) {
            self.add_or_update_lobby_player(msg.player_id.clone(), player.addr, player.info.username);
            self.ready_groups.retain(|g| !g.is_empty());
            self.refund_player(&msg.player_id);
            self.send_state();
        }
    }
}
