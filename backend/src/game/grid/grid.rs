//! Grid generation and tile breaking logic.
//!
//! This module provides functions to generate the game grid and to apply
//! the business rules for breaking or cracking tiles at each turn, depending
//! on the current game mode.

use crate::game::types::{Cell, Position, GameMode};
use crate::game::state::GameState;
use rand::seq::IteratorRandom;

/// Generate a new grid of the specified size, filled with solid tiles.
pub fn generate_grid(rows: usize, cols: usize) -> Vec<Vec<Cell>> {
    vec![vec![Cell::Solid; cols]; rows]
}

/// Apply the tile breaking logic for the current turn, depending on the game mode.
///
/// - In Classic mode: randomly select a solid tile and break it.
/// - In Cracked mode: Randomly select a solid tile and turn it into a cracked tile. + All cracked tiles become broken.
pub fn break_tile(game_state: &mut GameState) {
    let mut rng = rand::rng();

    match game_state.mode {
        GameMode::Classic => {
            // Collect all coordinates of solid tiles.
            let solid_tiles: Vec<(usize, usize)> = game_state.grid.iter().enumerate()
                .flat_map(|(y, row)| row.iter().enumerate().filter_map(move |(x, cell)| {
                    if *cell == Cell::Solid {
                        Some((y, x))
                    } else {
                        None
                    }
                }))
                .collect();

            // If there is at least one solid tile, randomly select one to break.
            if let Some(&(y, x)) = solid_tiles.iter().choose(&mut rng) {
                // If a cannonball is present on the tile to be broken, remove it.
                if let Some(pos) = game_state.cannonballs.iter().position(|c| c.pos == Position { x, y }) {
                    game_state.cannonballs.remove(pos);
                }
                // Mark the selected tile as broken.
                game_state.grid[y][x] = Cell::Broken;
            }
        }
        GameMode::Cracked => {
            // Step 1: All cracked tiles become broken.
            for row in game_state.grid.iter_mut() {
                for cell in row.iter_mut() {
                    if *cell == Cell::Cracked {
                        *cell = Cell::Broken;
                    }
                }
            }
            // Step 2: Collect all solid tiles to select one to crack.
            let solid_tiles: Vec<(usize, usize)> = game_state.grid.iter().enumerate()
                .flat_map(|(y, row)| row.iter().enumerate().filter_map(move |(x, cell)| {
                    if *cell == Cell::Solid {
                        Some((y, x))
                    } else {
                        None
                    }
                }))
                .collect();

            // If there is at least one solid tile, randomly select one to crack.
            if let Some(&(y, x)) = solid_tiles.iter().choose(&mut rng) {
                // If a cannonball is present on the tile to be cracked, remove it.
                if let Some(pos) = game_state.cannonballs.iter().position(|c| c.pos == Position { x, y }) {
                    game_state.cannonballs.remove(pos);
                }
                // Mark the selected tile as cracked.
                game_state.grid[y][x] = Cell::Cracked;
            }
        }
    }
}