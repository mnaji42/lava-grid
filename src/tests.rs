#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_game_over() {
        assert_eq!(check_game_over(), false);
    }
     fn test_generate_grid() {
        let rows = 10;
        let cols = 10;
        let grid = generate_grid(rows, cols);

        assert_eq!(grid.len(), rows);
        for row in grid {
            assert_eq!(row.len(), cols);
            assert!(row.iter().all(|&cell| cell == 1));
        }
    }
}