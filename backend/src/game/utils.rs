use super::types::{Cell};
use super::state::GameState;

pub fn resolve_cannonball_hits(game_state: &mut GameState) {
    for tile in &game_state.targeted_tiles {
        game_state.grid[tile.y][tile.x] = Cell::Broken;
    }
    game_state.targeted_tiles.clear();
}