use serde::{Serialize, Deserialize};
use rand::{Rng, rng};

use crate::game::types::{Player, Cell, Cannonball, TargetedTile, GameMode}; // Direction supprimé
use crate::game::grid::{generate_grid};
use crate::game::entities::{spawn_random_player, spawn_random_cannonballs, shoot_cannonball};
use crate::game::systems::{move_player, apply_rules, apply_player_rules};
use crate::server::game_session::messages::ClientAction;
use crate::server::matchmaking::types::PlayerInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub grid: Vec<Vec<Cell>>,
    pub players: Vec<Player>,
    pub cannonballs: Vec<Cannonball>,
    pub turn: u32,
    pub targeted_tiles: Vec<TargetedTile>,
    pub mode: GameMode,
}

impl GameState {
    // Crée un nouvel état de jeu
    pub fn new(rows: usize, cols: usize, player_infos: Vec<PlayerInfo>, mode: GameMode) -> Self {
        let mut players = vec![];

        for (i, info) in player_infos.iter().enumerate() {
            if let Some(player) = spawn_random_player(&generate_grid(rows, cols), &players, (i+1) as u8, info.username.clone()) {
                players.push(player);
            }
        }

        let nb_cannonballs = rng().random_range(1..=3);
        let cannonballs = spawn_random_cannonballs(&generate_grid(rows, cols), &players, 0, nb_cannonballs);

        GameState {
            grid: generate_grid(rows, cols),
            players,
            cannonballs,
            turn: 1,
            targeted_tiles: Vec::new(),
            mode,
        }
    }

    pub fn apply_player_action(&mut self, action: ClientAction, player_index: usize) {
        match action {
            ClientAction::Move(direction) => {
                move_player(self, player_index, direction);
            }
            ClientAction::Shoot { x, y } => {
                shoot_cannonball(self, player_index, x, y);
            }
        }
        apply_player_rules(self, player_index);
    }

    pub fn next_turn(&mut self) {
        apply_rules(self);
        for i in 0..self.players.len() {
            apply_player_rules(self, i);
        }
        self.turn += 1;
    }
}
