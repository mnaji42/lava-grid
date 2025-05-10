#[cfg(test)]
mod tests {
    use super::*;
    use crate::{grid::*, player::*, cannonball::*, types::*};

    #[test]
    fn test_grid_generation_size() {
        let grid = create_grid(10, 10);
        assert_eq!(grid.len(), 10);
        assert!(grid.iter().all(|row| row.len() == 10));
    }

    #[test]
    fn test_player_spawn_no_overlap() {
        let grid = create_grid(10, 10);
        let mut players = vec![];

        for id in 0..5 {
            let player = spawn_random_player(&grid, &players, id).expect("Failed to spawn player");
            assert!(!players.iter().any(|p| p.pos == player.pos));
            players.push(player);
        }
    }

    #[test]
    fn test_spawn_player_no_space() {
        let mut grid = vec![vec![Cell::Lava; 5]; 5]; // aucune case valide
        let players = vec![];
        let player = spawn_random_player(&grid, &players, 0);
        assert!(player.is_none());
    }

    #[test]
    fn test_cannonball_spawn_limit() {
        let grid = create_grid(5, 5);
        let players = vec![];
        let cannonballs = spawn_random_cannonballs(&grid, &players, 0, 100);
        assert!(cannonballs.len() <= 25); // au max 25 tiles solides
    }

    #[test]
    fn test_move_player_into_lava() {
        let mut grid = create_grid(5, 5);
        let mut player = Player::new(1, Position { x: 2, y: 2 });

        grid[2][3] = Cell::Lava;
        move_player(&mut player, Direction::Right, &grid);

        assert!(!player.is_alive);
    }

    #[test]
    fn test_pickup_cannonball() {
        let grid = create_grid(5, 5);
        let mut player = Player::new(1, Position { x: 2, y: 2 });
        let cannonball_pos = Position { x: 2, y: 3 };
        let mut cannonballs = vec![Cannonball { pos: cannonball_pos }];

        move_player(&mut player, Direction::Right, &grid);
        try_pickup_cannonball(&mut player, &mut cannonballs);

        assert_eq!(player.cannonball_count, 1);
        assert!(cannonballs.is_empty());
    }

    #[test]
    fn test_break_tile_replaces_with_lava() {
        let mut grid = create_grid(5, 5);
        let mut rng = rand::rng();

        break_tile(&mut grid, &mut rng);
        let lava_count = grid.iter().flatten().filter(|&&c| c == Cell::Lava).count();
        assert_eq!(lava_count, 1);
    }

    #[test]
    fn test_player_does_not_spawn_on_object() {
        let mut grid = create_grid(5, 5);
        let mut players = vec![];
        let mut cannonballs = vec![
            Cannonball { pos: Position { x: 1, y: 1 } },
            Cannonball { pos: Position { x: 2, y: 2 } },
        ];

        for id in 0..10 {
            if let Some(p) = spawn_random_player(&grid, &players, id) {
                // Le joueur ne spawn pas sur un cannonball
                assert!(!cannonballs.iter().any(|c| c.pos == p.pos));
                players.push(p);
            }
        }
    }
}
