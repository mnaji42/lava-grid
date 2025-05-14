use serde::{Serialize, Deserialize};
use rand::{Rng, rng};

use crate::game::types::{Player, Cell, Cannonball, TargetedTile, Direction};
use crate::game::grid::{generate_grid};
use crate::game::entities::{spawn_random_player, spawn_random_cannonballs, shoot_cannonball};
use crate::game::systems::{move_player, apply_rules};
use crate::server::game_session::messages::ClientAction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub grid: Vec<Vec<Cell>>,
    pub players: Vec<Player>,
    pub cannonballs: Vec<Cannonball>,
    pub turn: u32,
    pub targeted_tiles: Vec<TargetedTile>,
}

impl GameState {
    // Crée un nouvel état de jeu
    pub fn new(rows: usize, cols: usize, player_count: usize) -> Self {
        let mut players = vec![];
        for id in 1..=player_count {
            if let Some(player) = spawn_random_player(&generate_grid(rows, cols), &players, id as u8) {
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
            targeted_tiles: Vec::new()
        }
    }

    pub fn apply_player_action(&mut self, action: ClientAction, player_index: usize) {
        match action {
            ClientAction::Move(direction) => {
                move_player(self, player_index, direction);
                apply_rules(self, player_index);
            }
            ClientAction::Shoot { x, y } => {
                shoot_cannonball(self, player_index, x, y);
                // Optionnel: appliquer les règles si nécessaire après un tir
                apply_rules(self, player_index);
            }
        }
        self.next_turn();
    }

    // Passe au tour suivant
    pub fn next_turn(&mut self) {
        self.turn += 1;
    }
}
