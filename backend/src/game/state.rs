//! Game state structure and core logic.
//!
//! This module defines the main GameState struct, which tracks the grid, players,
//! cannonballs, turn number, and targeted tiles. It also provides methods to
//! initialize the game, apply player actions, and advance turns.

use serde::{Serialize, Deserialize};
use rand::{Rng, rng};

use crate::game::types::{Player, Cell, Cannonball, TargetedTile, GameMode};
use crate::game::grid::generate_grid;
use crate::game::entities::{spawn_random_player, spawn_random_cannonballs, shoot_cannonball};
use crate::game::systems::{move_player, apply_rules, apply_player_rules};
use crate::server::game_session::messages::PlayerAction;
use crate::server::matchmaking::types::PlayerInfo;

/// Represents the full state of a running game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// 2D grid of cells.
    pub grid: Vec<Vec<Cell>>,
    /// All players in the game.
    pub players: Vec<Player>,
    /// All cannonballs currently on the grid.
    pub cannonballs: Vec<Cannonball>,
    /// Current turn number (starts at 1).
    pub turn: u32,
    /// Tiles targeted by cannonball shots this turn.
    pub targeted_tiles: Vec<TargetedTile>,
    /// Current game mode.
    pub mode: GameMode,
}

impl GameState {
    /// Create a new game state with the given grid size, player infos, and mode.
    pub fn new(rows: usize, cols: usize, player_infos: Vec<PlayerInfo>, mode: GameMode) -> Self {
        // Generate the grid only once
        let grid = generate_grid(rows, cols);
        
        let mut players = vec![];

        // Spawn each player at a random valid position.
        for (i, info) in player_infos.iter().enumerate() {
            if let Some(player) = spawn_random_player(&grid, &players, (i+1) as u8, info.username.clone()) {
                players.push(player);
            }
        }

        // Initialize an empty cannonballs list
        let cannonballs_list: Vec<Cannonball> = Vec::new();
        
        // Randomly determine the number of cannonballs to spawn (1 to 3).
        let nb_cannonballs = rng().random_range(1..=3);
        let cannonballs = spawn_random_cannonballs(&grid, &players, &cannonballs_list, nb_cannonballs);

        GameState {
            grid,
            players,
            cannonballs,
            turn: 1,
            targeted_tiles: Vec::new(),
            mode,
        }
    }

    /// Apply a player action (move or shoot) for the given player index.
    pub fn apply_player_action(&mut self, action: PlayerAction, player_index: usize) {
        match action {
            PlayerAction::Move(direction) => {
                // Move the player in the specified direction.
                move_player(self, player_index, direction);
            }
            PlayerAction::Shoot { x, y } => {
                // Attempt to shoot a cannonball at the specified tile.
                shoot_cannonball(self, player_index, x, y);
            }
        }
        // Apply rules that affect only this player (e.g., pickup, death).
        apply_player_rules(self, player_index);
    }

    /// Advance to the next turn, applying global and per-player rules.
    pub fn next_turn(&mut self) {
        // Apply global rules (e.g., break a tile, resolve cannonball hits).
        apply_rules(self);
        // Apply per-player rules (e.g., check for death, pickup).
        for i in 0..self.players.len() {
            apply_player_rules(self, i);
        }
        self.turn += 1;
    }
}
