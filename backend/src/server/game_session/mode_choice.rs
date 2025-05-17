/// Handles the game mode choice phase for a GameSession.
/// Encapsulates voting, broadcasting pre-game data, and finalization logic.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use actix::prelude::*;
use rand::prelude::IteratorRandom;
use log::info;

use crate::game::types::GameMode;
use crate::server::matchmaking::types::{PlayerInfo, WalletAddress};
use crate::server::game_session::messages::{
    GamePreGameData, GameModeVoteUpdate, GameModeChosen,
};
use crate::config::game::{MODE_CHOICE_DURATION, GRID_ROW, GRID_COL};
use crate::server::game_session::session::GameSessionActor;

/// Represents the state and logic for the mode choice phase.
pub struct ModeChoice {
    pub votes: HashMap<WalletAddress, GameMode>,
    pub deadline: Instant,
    pub timer: Option<SpawnHandle>,
    pub chosen_mode: Option<GameMode>,
    pub chosen_by: Option<WalletAddress>,
    pub required_players: usize,
}

impl ModeChoice {
    /// Create a new ModeChoice phase for the given number of players.
    pub fn new(required_players: usize) -> Self {
        Self {
            votes: HashMap::new(),
            deadline: Instant::now() + Duration::from_secs(MODE_CHOICE_DURATION),
            timer: None,
            chosen_mode: None,
            chosen_by: None,
            required_players,
        }
    }

    /// Broadcast pre-game data (available modes, deadline, players, grid size) to all sessions.
    pub fn broadcast_to_players_pre_game_data(
        &self,
        players: &std::collections::HashMap<WalletAddress, Addr<GameSessionActor>>,
        spectators: &std::collections::HashMap<WalletAddress, Addr<GameSessionActor>>,
        player_infos: &[PlayerInfo],
    ) {
        let deadline_secs = self.deadline.saturating_duration_since(Instant::now()).as_secs();
        let msg = GamePreGameData {
            modes: vec![GameMode::Classic, GameMode::Cracked],
            deadline_secs,
            players: player_infos.to_vec(),
            grid_row: GRID_ROW,
            grid_col: GRID_COL,
        };
        for addr in players.values().chain(spectators.values()) {
            addr.do_send(msg.clone());
        }
    }

    /// Register a mode vote from a player and broadcast the update.
    pub fn receive_mode_vote(
        &mut self,
        player_id: WalletAddress,
        mode: GameMode,
        players: &std::collections::HashMap<WalletAddress, Addr<GameSessionActor>>,
        spectators: &std::collections::HashMap<WalletAddress, Addr<GameSessionActor>>,
    ) -> bool {
        self.votes.insert(player_id.clone(), mode.clone());
        let vote_update = GameModeVoteUpdate {
            player_id,
            mode,
        };
        for addr in players.values().chain(spectators.values()) {
            addr.do_send(vote_update.clone());
        }
        // Return true if all players have voted.
        self.votes.len() >= self.required_players
    }

    /// Finalize the mode choice, picking randomly if needed.
    pub fn finalize_mode_choice(
        &mut self,
        player_infos: &[PlayerInfo],
        players: &std::collections::HashMap<WalletAddress, Addr<GameSessionActor>>,
        spectators: &std::collections::HashMap<WalletAddress, Addr<GameSessionActor>>,
    ) {
        let (chosen_mode, chosen_by) = if !self.votes.is_empty() {
            let mut rng = rand::rng();
            let (chosen_player, mode) = self.votes.iter().choose(&mut rng).unwrap();
            (mode.clone(), chosen_player.clone())
        } else {
            let modes = [GameMode::Classic, GameMode::Cracked];
            let mut rng = rand::rng();
            let mode = *modes.iter().choose(&mut rng).unwrap();
            let chosen_player = player_infos.iter().choose(&mut rng).unwrap().id.clone();
            (mode, chosen_player)
        };
        self.chosen_mode = Some(chosen_mode.clone());
        self.chosen_by = Some(chosen_by.clone());
        for addr in players.values().chain(spectators.values()) {
            addr.do_send(GameModeChosen {
                mode: chosen_mode.clone(),
                chosen_by: chosen_by.clone(),
            });
        }
        info!("[ModeChoice] Mode chosen: {:?} by {}", chosen_mode, chosen_by);
    }

    /// Reset the mode choice phase (for restarts).
    pub fn reset(&mut self) {
        self.votes.clear();
        self.chosen_mode = None;
        self.chosen_by = None;
        self.deadline = Instant::now() + Duration::from_secs(MODE_CHOICE_DURATION);
        self.timer = None;
    }
}