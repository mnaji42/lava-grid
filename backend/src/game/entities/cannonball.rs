use super::types::{Player, Cell, Position, Cannonball, TargetedTile};
use super::state::GameState;
use rand::seq::IteratorRandom;

pub fn spawn_random_cannonballs(
    grid: &Vec<Vec<Cell>>,
    players: &Vec<Player>,
    existing_cannonballs: usize,
    count: usize,
) -> Vec<Cannonball> {
    let mut rng = rand::rng();

    let valid_positions: Vec<Position> = grid.iter().enumerate()
        .flat_map(|(x, row)| row.iter().enumerate().filter_map(move |(y, cell)| {
            if *cell == Cell::Solid {
                Some(Position { x, y })
            } else {
                None
            }
        }))
        .collect();

    let occupied_positions: Vec<Position> = players.iter().map(|p| p.pos)
        .chain(std::iter::repeat(Position { x: 0, y: 0 }).take(existing_cannonballs)) // Faux pour l’instant
        .collect();

    let free_positions: Vec<Position> = valid_positions
        .into_iter()
        .filter(|pos| !occupied_positions.contains(pos))
        .collect();

    if free_positions.len() == 0 {
        println!("[WARN] Aucune case libre pour placer un boulet !");
        return vec![];
    }

    free_positions.iter()
        .choose_multiple(&mut rng, count.min(free_positions.len()))
        .into_iter()
        .map(|pos| Cannonball { pos: *pos })
        .collect()
}

pub fn shoot_cannonball(game_state: &mut GameState, player_id: usize, x: usize, y: usize) {
    let player = &mut game_state.players[player_id];
    if player.cannonball_count > 0 {
        if !game_state.targeted_tiles.iter().any(|t| t.x == x && t.y == y) {
            game_state.targeted_tiles.push(TargetedTile { x, y });
            player.cannonball_count -= 1;
        }
    }
}