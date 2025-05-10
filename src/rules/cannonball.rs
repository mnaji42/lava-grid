use crate::rules::types::{Player, Cell};
use rand::{seq::IteratorRandom, rngs::ThreadRng, Rng};

// Function to place cannonballs randomly on the grid
pub fn generate_cannonballs(grid: &mut Vec<Vec<Cell>>) {
    let mut rng: ThreadRng = rand::rng();

    // Choose a random number of cannonballs to place (between 1 and 3)
    let num_cannonballs = rng.random_range(1..=3);

    // Create a list of all available positions (solid cells)
    let mut available_positions = Vec::new();
    for i in 0..grid.len() {
        for j in 0..grid[i].len() {
            if matches!(grid[i][j], Cell::Solid) {
                available_positions.push((i, j));
            }
        }
    }

    // Place the cannonballs randomly on the grid
    for _ in 0..num_cannonballs {
        if let Some((x, y)) = available_positions.iter().choose(&mut rng) {
            grid[*x][*y] = Cell::Cannonball;
        }
    }
}

// Function to use a cannonball (destroy a specific cell)
pub fn use_cannonball(player: &mut Player, x: usize, y: usize, grid: &mut Vec<Vec<Cell>>) {
    if player.has_cannonball {
        // Destroy the chosen cell by setting it to `Cell::Broken`
        if x < grid.len() && y < grid[0].len() {
            grid[x][y] = Cell::Broken;
            player.has_cannonball = false; // Once used, the cannonball is consumed
        }
    }
}
