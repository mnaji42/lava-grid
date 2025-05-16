use crate::game::types::{Cell};
use crate::game::state::GameState;
use crate::game::grid::break_tile;
use crate::game::utils::resolve_cannonball_hits;

pub fn apply_player_rules(game_state: &mut GameState, player_index: usize) {

    let player = &mut game_state.players[player_index];

    // Try to pick up cannonball
    if let Some(pos) = game_state.cannonballs.iter().position(|c| c.pos == player.pos) {
        player.cannonball_count += 1;
        game_state.cannonballs.remove(pos);
    }

    // Check if there is a tile at the new pos
    if game_state.grid[player.pos.y][player.pos.x] == Cell::Broken {
        player.is_alive = false;
    }
}

pub fn apply_rules(game_state: &mut GameState) {
    break_tile(game_state);
    resolve_cannonball_hits(game_state);
}
