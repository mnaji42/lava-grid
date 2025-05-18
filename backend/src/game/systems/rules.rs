//! Game rules system.
//!
//! This module applies per-player and global rules at each turn.

use crate::game::types::Cell;
use crate::game::state::GameState;
use crate::game::grid::break_tile;
use crate::game::utils::resolve_cannonball_hits;

/// Apply rules that affect a single player (e.g., pickup, death).
pub fn apply_player_rules(game_state: &mut GameState, player_index: usize) {
    let player = &mut game_state.players[player_index];

    // If the player is on a cannonball, pick it up.
    if let Some(pos) = game_state.cannonballs.iter().position(|c| c.pos == player.pos) {
        player.cannonball_count += 1;
        game_state.cannonballs.remove(pos);
    }

    // Check grid bounds before accessing the cell.
    let grid_height = game_state.grid.len();
    let grid_width = if grid_height > 0 { game_state.grid[0].len() } else { 0 };

    if player.pos.y < grid_height && player.pos.x < grid_width {
        // If the player is on a broken tile, they die.
        if game_state.grid[player.pos.y][player.pos.x] == Cell::Broken {
            player.is_alive = false;
        }
    } else {
        // If the player is out of bounds, they die.
        player.is_alive = false;
    }
}

/// Apply global rules at the end of the turn (e.g., break a tile, resolve cannonball hits).
pub fn apply_rules(game_state: &mut GameState) {
    break_tile(game_state); // TODO remettre (juste debugging)
    resolve_cannonball_hits(game_state);
}