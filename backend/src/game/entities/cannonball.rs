//! Cannonball entity logic.
//!
//! This module handles spawning cannonballs and shooting them.

use crate::game::types::{Player, Cell, Position, Cannonball, TargetedTile};
use crate::game::state::GameState;
use rand::seq::IteratorRandom;

/// Spawn random cannonballs on valid grid positions.
/// Ensures cannonballs do not overlap with players or other cannonballs.
pub fn spawn_random_cannonballs(
    grid: &Vec<Vec<Cell>>,
    players: &Vec<Player>,
    existing_cannonballs: usize,
    count: usize,
) -> Vec<Cannonball> {
    let mut rng = rand::rng();

    // Collect all solid tiles.
    let valid_positions: Vec<Position> = grid.iter().enumerate()
        .flat_map(|(x, row)| row.iter().enumerate().filter_map(move |(y, cell)| {
            if *cell == Cell::Solid {
                Some(Position { x, y })
            } else {
                None
            }
        }))
        .collect();

    // Mark positions occupied by players and existing cannonballs.
    let occupied_positions: Vec<Position> = players.iter().map(|p| p.pos)
        .chain(std::iter::repeat(Position { x: 0, y: 0 }).take(existing_cannonballs)) // TODO: Fix for real cannonball positions
        .collect();

    // Only keep positions not occupied.
    let free_positions: Vec<Position> = valid_positions
        .into_iter()
        .filter(|pos| !occupied_positions.contains(pos))
        .collect();

    if free_positions.len() == 0 {
        println!("[WARN] No free tile to place a cannonball!");
        return vec![];
    }

    // Randomly select positions for new cannonballs.
    free_positions.iter()
        .choose_multiple(&mut rng, count.min(free_positions.len()))
        .into_iter()
        .map(|pos| Cannonball { pos: *pos })
        .collect()
}

/// Attempt to shoot a cannonball at the specified tile for the given player.
/// Only succeeds if the player has at least one cannonball and the tile is not already targeted.
pub fn shoot_cannonball(game_state: &mut GameState, player_id: usize, x: usize, y: usize) {
    let player = &mut game_state.players[player_id];
    if player.cannonball_count > 0 {
        // Only allow shooting if the tile is not already targeted this turn.
        if !game_state.targeted_tiles.iter().any(|t| t.x == x && t.y == y) {
            game_state.targeted_tiles.push(TargetedTile { x, y });
            player.cannonball_count -= 1;
        }
    }
}