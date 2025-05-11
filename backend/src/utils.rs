use crate::types::{Cell};
use crate::state::GameState;

pub fn resolve_cannonball_hits(game_state: &mut GameState) {
    for tile in &game_state.targeted_tiles {
        game_state.grid[tile.x][tile.y] = Cell::Broken;
    }
    game_state.targeted_tiles.clear();
}