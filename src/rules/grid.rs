use crate::rules::types::{Cell}; 

// Generates a grid of size (rows x cols), filled with '1' (solid)
pub fn generate_grid(rows: usize, cols: usize) -> Vec<Vec<Cell>> {
    // Fill the grid with solid cells
    vec![vec![Cell::Solid; cols]; rows]
}