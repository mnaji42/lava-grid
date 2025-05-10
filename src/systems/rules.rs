use crate::types::{Cell};
use crate::state::GameState;

pub fn apply_rules(game_state: &mut GameState, player_index: usize) {

    let player = &mut game_state.players[player_index];
    // Check if there is a tile at the new pos
    if game_state.grid[player.pos.x][player.pos.y] == Cell::Broken {
        player.is_alive = false;
    }

    // Try to picj up cannonball
    if let Some(pos) = game_state.cannonballs.iter().position(|c| c.pos == player.pos) {
        player.cannonball_count += 1;
        game_state.cannonballs.remove(pos);
    }
}
