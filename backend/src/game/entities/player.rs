//! Player entity logic.
//!
//! This module handles spawning players at random valid positions.

use crate::game::types::{Player, Cell, Position};
use rand::seq::IteratorRandom;

/// Generate a new player at a random valid position on the grid.
/// Returns None if no valid position is available.
pub fn spawn_random_player(
    grid: &Vec<Vec<Cell>>,
    players: &Vec<Player>,
    id: u8,
    username: String,
) -> Option<Player> {
    let mut rng = rand::rng();

    // Collect all solid tiles not already occupied by another player.
    let valid_positions: Vec<Position> = grid.iter().enumerate()
        .flat_map(|(x, row)| {
            row.iter().enumerate().filter_map(move |(y, cell)| {
                if *cell == Cell::Solid && !players.iter().any(|p| p.pos.x == x && p.pos.y == y) {
                    Some(Position { x, y })
                } else {
                    None
                }
            })
        })
        .collect();

    if valid_positions.len() == 0 {
        println!("[WARN] Cannot place player {}: no free tile.", id);
        return None;
    }

    // Randomly select a valid position.
    valid_positions.into_iter()
        .choose(&mut rng)
        .map(|pos| Player::new(id, pos, username))
}