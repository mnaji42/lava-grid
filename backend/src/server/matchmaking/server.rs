/// Matchmaking server actor.
///
/// Manages the matchmaking lobby, player readiness, countdowns, and game creation.
/// Handles player join/leave, payment, and cancellation, and coordinates with the game session manager.

use actix::prelude::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;
use log::{info, debug};

use super::types::{PlayerInfo, WalletAddress};
use super::messages::{ServerWsMessage, MatchmakingState};
use super::session::MatchmakingSession;
use crate::config::matchmaking::{MIN_PLAYERS, MAX_PLAYERS, COUNTDOWN_DURATION_SECS};
use crate::server::game_session::messages::RegisterPendingGame;
use crate::server::game_session::server::GameSessionManager;

type SessionAddr = Addr<MatchmakingSession>;

use crate::server::matchmaking::messages::SessionKicked;


/// Represents a player currently connected to the lobby or ready group.
#[derive(Debug, Clone)]
struct ConnectedPlayer {
    info: PlayerInfo,
    addr: SessionAddr,
}

/// Handle for an active countdown timer.
struct CountdownHandle {
    handle: SpawnHandle,
    start_time: Instant,
}

/// Main matchmaking server actor.
pub struct MatchmakingServer {
    /// Players in the lobby (not yet ready).
    lobby_players: HashMap<WalletAddress, ConnectedPlayer>,
    /// Groups of players who have paid and are ready to play.
    ready_groups: Vec<HashMap<WalletAddress, ConnectedPlayer>>,
    /// Active countdown timer, if any.
    countdown: Option<CountdownHandle>,
    /// Address of the game session manager for launching games.
    game_session_manager: Addr<GameSessionManager>,
}

impl MatchmakingServer {
    /// Create a new matchmaking server.
    pub fn new(game_session_manager: Addr<GameSessionManager>) -> Self {
        Self {
            lobby_players: HashMap::new(),
            ready_groups: Vec::new(),
            countdown: None,
            game_session_manager,
        }
    }

    /// Broadcast a message to all players in the lobby and ready groups.
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

    /// Send the current matchmaking state to all clients.
    fn send_state(&self) {
        let state = self.get_state();
        self.broadcast(ServerWsMessage::UpdateState(state));
    }

    /// Build the current matchmaking state.
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

    /// Start the countdown timer for the next game if not already started.
    fn start_countdown(&mut self, ctx: &mut Context<Self>) {
        if self.countdown.is_some() {
            // Countdown already active; do nothing.
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

    /// Cancel the countdown timer if active.
    fn cancel_countdown(&mut self, ctx: &mut Context<Self>) {
        if let Some(countdown) = self.countdown.take() {
            ctx.cancel_future(countdown.handle);
            self.send_state();
            info!("[Matchmaking] Countdown cancelled");
        }
    }

    /// Attempt to launch the next game if a ready group has enough players.
    fn try_launch_next_game(&mut self, ctx: &mut Context<Self>) {
        // Find a group with enough players to start a game.
        if let Some((group_idx, group)) = self.ready_groups.iter().enumerate().find(|(_, g)| g.len() >= MIN_PLAYERS) {
            let player_infos: Vec<PlayerInfo> = group.values().map(|p| p.info.clone()).collect();
            let player_addrs: Vec<SessionAddr> = group.values().map(|p| p.addr.clone()).collect();

            // Remove the countdown since the game is starting.
            self.cancel_countdown(ctx);

            // Generate a new game ID.
            let game_id = Uuid::new_v4();

            // Register the pending game with the game session manager.
            self.game_session_manager.do_send(RegisterPendingGame {
                game_id,
                players: player_infos.clone(),
            });

            // Notify each player of the new game.
            for addr in &player_addrs {
                addr.do_send(ServerWsMessage::GameStarted { game_id });
            }

            info!("[Matchmaking] Game created with {} players, game_id={}", player_addrs.len(), game_id);

            // Remove the group from the ready list.
            if group_idx < self.ready_groups.len() {
                self.ready_groups.remove(group_idx);
            }
            self.send_state();
        }
    }

    /// Add or update a player in the lobby.
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

    /// Remove a player from all ready groups, but only if the session address matches.
    fn remove_player_from_ready_groups(&mut self, player_id: &WalletAddress, addr: &SessionAddr) -> Option<ConnectedPlayer> {
        for group in &mut self.ready_groups {
            if let Some(player) = group.get(player_id) {
                if &player.addr == addr {
                    return group.remove(player_id);
                }
            }
        }
        None
    }

    /// Find the ready group containing the given player, mutably.
    fn find_group_of_player_mut(&mut self, player_id: &WalletAddress) -> Option<&mut HashMap<WalletAddress, ConnectedPlayer>> {
        self.ready_groups.iter_mut().find(|g| g.contains_key(player_id))
    }

    /// Refund a player (stub for business logic).
    fn refund_player(&self, player_id: &WalletAddress) {
        // TODO: Implement refund logic if needed.
        debug!("[Matchmaking] Refund requested for player {}", player_id);
    }
}

/// Message: player joins the lobby.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub player_id: WalletAddress,
    pub addr: SessionAddr,
    pub username: String,
}

/// Message: player leaves the lobby or ready group.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Leave {
    pub player_id: WalletAddress,
    pub addr: SessionAddr,
}

/// Message: player pays to become ready.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Pay {
    pub player_id: WalletAddress,
    pub addr: SessionAddr,
}

/// Message: player cancels payment and returns to lobby.
#[derive(Message)]
#[rtype(result = "()")]
pub struct CancelPayment {
    pub player_id: WalletAddress,
    pub addr: SessionAddr,
}

impl Actor for MatchmakingServer {
    type Context = Context<Self>;
}

impl Handler<Join> for MatchmakingServer {
    type Result = ();

    /// Handles a player joining the lobby.
    fn handle(&mut self, msg: Join, _ctx: &mut Self::Context) -> Self::Result {
        // If the player is already in a ready group, kick the old session and update their address.
        if let Some(group) = self.find_group_of_player_mut(&msg.player_id) {
            if let Some(player) = group.get_mut(&msg.player_id) {
                if player.addr != msg.addr {
                    // Kick the old session before replacing
                    player.addr.do_send(SessionKicked {
                        reason: "Another session has connected with your wallet.".to_string(),
                    });
                    player.addr = msg.addr.clone();
                    debug!("[Matchmaking] Player {} reconnected in ready_groups (old session kicked)", msg.player_id);
                    self.send_state();
                }
                return;
            }
        }
        // If the player is already in the lobby, kick the old session and update their address.
        if let Some(player) = self.lobby_players.get_mut(&msg.player_id) {
            if player.addr != msg.addr {
                // Kick the old session before replacing
                player.addr.do_send(SessionKicked {
                    reason: "Another session has connected with your wallet.".to_string(),
                });
                player.addr = msg.addr.clone();
                debug!("[Matchmaking] Player {} reconnected in lobby_players (old session kicked)", msg.player_id);
                self.send_state();
            }
            return;
        }
        // Otherwise, add as a new player in the lobby.
        self.add_or_update_lobby_player(msg.player_id.clone(), msg.addr, msg.username);
        debug!("[Matchmaking] Player {} joined lobby_players", msg.player_id);
        self.send_state();
    }
}

impl Handler<Leave> for MatchmakingServer {
    type Result = ();

    /// Handles a player leaving the lobby or ready group.
    fn handle(&mut self, msg: Leave, _ctx: &mut Self::Context) -> Self::Result {
        // Remove from lobby if present and session matches.
        if let Some(player) = self.lobby_players.get(&msg.player_id) {
            if player.addr == msg.addr {
                self.lobby_players.remove(&msg.player_id);
                debug!("[Matchmaking] Player {} left lobby_players", msg.player_id);
                self.send_state();
            }
            return;
        }

        let countdown_active = self.countdown.is_some();
        // If in a ready group, handle leave logic.
        if let Some(group) = self.find_group_of_player_mut(&msg.player_id) {
            if let Some(player) = group.get(&msg.player_id) {
                if player.addr != msg.addr {
                    // Not the same session, ignore.
                    return;
                }
            }
            
            if countdown_active {
                // Players cannot leave during countdown (game is about to start).
                debug!("[Matchmaking] Player {} tried to leave during countdown (not allowed)", msg.player_id);
                // TODO: send error message to client if needed.
                return;
            }
            
            group.remove(&msg.player_id);
            debug!("[Matchmaking] Player {} left ready_groups (removed, not put back in lobby)", msg.player_id);
            // Remove empty groups.
            self.ready_groups.retain(|g| !g.is_empty());
            self.refund_player(&msg.player_id);
            self.send_state();
            return;
        }
    }
}

impl Handler<Pay> for MatchmakingServer {
    type Result = ();

    /// Handles a player paying to become ready.
    fn handle(&mut self, msg: Pay, ctx: &mut Self::Context) -> Self::Result {
        // If already in a ready group, ignore (cannot pay twice).
        if let Some(group) = self.find_group_of_player_mut(&msg.player_id) {
            if let Some(player) = group.get(&msg.player_id) {
                if player.addr == msg.addr {
                    debug!("[Matchmaking] Player {} tried to pay but is already ready (same session)", msg.player_id);
                    // TODO: send error message to client if needed.
                    return;
                }
            }
        }
        
        // Remove from lobby; only if session matches.
        let player = match self.lobby_players.get(&msg.player_id) {
            Some(p) if p.addr == msg.addr => self.lobby_players.remove(&msg.player_id).unwrap(),
            _ => {
                debug!("[Matchmaking] Player {} tried to pay but is not in lobby_players or session mismatch", msg.player_id);
                // TODO: send error message to client if needed.
                return;
            }
        };

        // Try to add to an existing group with space.
        let mut added_to_group = false;
        for group in &mut self.ready_groups {
            if group.len() < MAX_PLAYERS {
                group.insert(msg.player_id.clone(), player.clone());
                added_to_group = true;
                break;
            }
        }
        // If no group has space, create a new group.
        if !added_to_group {
            let mut new_group = HashMap::new();
            new_group.insert(msg.player_id.clone(), player.clone());
            self.ready_groups.push(new_group);
        }

        debug!("[Matchmaking] Player {} moved to ready_groups", msg.player_id);

        // If the first group is full, launch the game immediately.
        if let Some(first_group) = self.ready_groups.first() {
            if first_group.len() >= MAX_PLAYERS {
                self.cancel_countdown(ctx);
                self.try_launch_next_game(ctx);
            } else if first_group.len() >= MIN_PLAYERS && self.countdown.is_none() {
                // If enough players for a game, but not full, start countdown.
                self.start_countdown(ctx);
            }
        }
        self.send_state();
    }
}

impl Handler<CancelPayment> for MatchmakingServer {
    type Result = ();

    /// Handles a player cancelling payment and returning to the lobby.
    fn handle(&mut self, msg: CancelPayment, _ctx: &mut Self::Context) -> Self::Result {
        let countdown_active = self.countdown.is_some();
        let group = self.find_group_of_player_mut(&msg.player_id);
        if group.is_none() {
            // Player is not in any ready group; nothing to do.
            return;
        }
        let group = group.unwrap();

        // Check if the session matches
        if let Some(player) = group.get(&msg.player_id) {
            if player.addr != msg.addr {
                // Not the same session, ignore.
                return;
            }
        }

        if countdown_active {
            // Cannot cancel payment during countdown (game is about to start).
            if let Some(player) = group.get(&msg.player_id) {
                player.addr.do_send(ServerWsMessage::Error {
                    message: "Cannot cancel payment: game is about to start.".to_string(),
                });
            }
            return;
        }

        // Remove from ready group and put back in lobby.
        if let Some(player) = group.remove(&msg.player_id) {
            self.add_or_update_lobby_player(msg.player_id.clone(), player.addr, player.info.username);
            // Remove empty groups.
            self.ready_groups.retain(|g| !g.is_empty());
            self.refund_player(&msg.player_id);
            self.send_state();
        }
    }
}
