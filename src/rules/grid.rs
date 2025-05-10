use rand::{seq::IteratorRandom, rngs::ThreadRng};
use crate::rules::types::{Cell}; 

// Generates a grid of size (rows x cols), filled with '1' (solid)
pub fn generate_grid(rows: usize, cols: usize) -> Vec<Vec<Cell>> {
    // Fill the grid with solid cells
    vec![vec![Cell::Solid; cols]; rows]
}

// Broque ramdomly a tile
pub fn break_tile(grid: &mut Vec<Vec<Cell>>) {
    let mut rng = rand::thread_rng();

    // Found all the solid tiles
    let solid_tiles: Vec<(usize, usize)> = grid.iter()
        .enumerate()
        .flat_map(|(x, row)| {
            row.iter().enumerate()
                .filter_map(move |(y, cell)| {
                    if matches!(cell, Cell::Solid) {
                        Some((x, y))
                    } else {
                        None
                    }
                })
        })
        .collect();

    // Broke one ramdomly
    if let Some(&(x, y)) = solid_tiles.iter().choose(&mut rng) {
        grid[x][y] = Cell::Broken;
        println!("Tile at ({}, {}) broke!", x, y);
    }
}