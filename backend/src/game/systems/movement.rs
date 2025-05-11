use crate::game::types::{Direction, Position};
use crate::game::state::GameState;

pub fn move_player(game_state: &mut GameState, player_index: usize, direction: Direction) -> Position {
    let player = &mut game_state.players[player_index];
    let mut new_pos = player.pos;

    match direction {
        Direction::Up => if new_pos.x > 0 { new_pos.x -= 1; },
        Direction::Down => if new_pos.x < game_state.grid.len() - 1 { new_pos.x += 1; },
        Direction::Left => if new_pos.y > 0 { new_pos.y -= 1; },
        Direction::Right => if new_pos.y < game_state.grid[0].len() - 1 { new_pos.y += 1; },
        Direction::Stay => {}
    }

    player.pos = new_pos;
    new_pos
}

