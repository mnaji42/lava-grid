use crate::types::{Cell};
use crate::state::GameState;
use crate::grid::grid::{break_tile};
use crate::utils::{resolve_cannonball_hits};

pub fn apply_rules(game_state: &mut GameState, player_index: usize) {

    break_tile(game_state);
    resolve_cannonball_hits(game_state);
    let player = &mut game_state.players[player_index];

    // Try to pick up cannonball
    if let Some(pos) = game_state.cannonballs.iter().position(|c| c.pos == player.pos) {
        player.cannonball_count += 1;
        game_state.cannonballs.remove(pos);
    }

    // Check if there is a tile at the new pos
    if game_state.grid[player.pos.x][player.pos.y] == Cell::Broken {
        player.is_alive = false;
    }
}
