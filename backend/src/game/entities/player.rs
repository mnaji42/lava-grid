use crate::game::types::{Player, Cell, Position};
use rand::seq::IteratorRandom;

/// Generate new player with random position
pub fn spawn_random_player(
    grid: &Vec<Vec<Cell>>,
    players: &Vec<Player>,
    id: u8,
    username: String,
) -> Option<Player> {
    let mut rng = rand::rng();

    let valid_positions: Vec<Position> = grid.iter().enumerate()
        .flat_map(|(x, row)| {
            row.iter().enumerate().filter_map(move |(y, cell)| {
                if *cell == Cell::Solid && !players.iter().any(|p| p.pos.x == x && p.pos.y == y) {
                    Some(Position { x, y })
                } else {
                    None
                }
            })
        })
        .collect();

    if valid_positions.len() == 0 {
        println!("[WARN] Impossible de placer le joueur {} : aucune case libre.", id);
        return None;
    }

    valid_positions.into_iter()
        .choose(&mut rng)
        .map(|pos| Player::new(id, pos, username))
}