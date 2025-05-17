//! Game utility functions.
//!
//! This module provides helper functions for game state updates.

use super::types::Cell;
use super::state::GameState;

/// Resolve all cannonball hits for the current turn.
/// Breaks all targeted tiles and clears the targeted list.
pub fn resolve_cannonball_hits(game_state: &mut GameState) {
    for tile in &game_state.targeted_tiles {
        // Mark the targeted tile as broken.
        game_state.grid[tile.y][tile.x] = Cell::Broken;
    }
    // Clear the list of targeted tiles for the next turn.
    game_state.targeted_tiles.clear();
}