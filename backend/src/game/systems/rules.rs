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

    // Vérification des bornes avant d'accéder à la grille
    let grid_height = game_state.grid.len();
    let grid_width = if grid_height > 0 { game_state.grid[0].len() } else { 0 };

    if player.pos.y < grid_height && player.pos.x < grid_width {
        if game_state.grid[player.pos.y][player.pos.x] == Cell::Broken {
            player.is_alive = false;
        }
    } else {
        // Position invalide, on considère le joueur comme mort (hors de la grille)
        player.is_alive = false;
    }
}

pub fn apply_rules(game_state: &mut GameState) {
    break_tile(game_state);
    resolve_cannonball_hits(game_state);
}
