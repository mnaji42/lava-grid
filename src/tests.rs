#[cfg(test)]
mod tests {
    use super::*; // This imports everything from the current module

    #[test]
    fn test_generate_grid() {
        let rows = 10;
        let cols = 10;
        let grid = generate_grid(rows, cols);

        // Ensure the grid has the correct dimensions
        assert_eq!(grid.len(), rows);
        for row in &grid {
            assert_eq!(row.len(), cols);
            // Ensure all cells are `Cell::Solid`
            assert!(row.iter().all(|&cell| matches!(cell, Cell::Solid)));
        }
    }

    #[test]
    fn test_initialize_players() {
        let mut grid = generate_grid(10, 10);
        let players = initialize_players(&mut grid, 5);

        // Verify that the correct number of players has been initialized
        assert_eq!(players.len(), 5);

        // Ensure the players are placed on solid tiles (`Cell::Player`)
        for player in &players {
            assert_eq!(grid[player.pos.x][player.pos.y], Cell::Player(player.id));
        }

        // Ensure players are placed on distinct positions
        let mut positions = Vec::new();
        for player in &players {
            let pos = (player.pos.x, player.pos.y);
            assert!(!positions.contains(&pos), "Player {} is placed on a duplicate position", player.id);
            positions.push(pos);
        }
    }

    #[test]
    fn test_random_player_positions() {
        let mut grid = generate_grid(10, 10);
        let players = initialize_players(&mut grid, 5);

        // Verify that players' positions are random and distinct
        let mut positions = Vec::new();
        for player in &players {
            let pos = (player.pos.x, player.pos.y);
            // Check that the position is unique
            assert!(!positions.contains(&pos), "Player {} is placed on a duplicate position", player.id);
            positions.push(pos);
        }

        // Ensure players are not placed on the same cell
        assert_eq!(positions.len(), players.len(), "Some players share the same position");
    }
}
