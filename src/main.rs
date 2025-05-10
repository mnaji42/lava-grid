mod rules;
mod rules_dev;

use crate::rules::{generate_grid, initialize_players, generate_cannonballs};
use crate::rules_dev::print_grid;

fn main() {
    let mut grid = generate_grid(10, 10);
    let players = initialize_players(&mut grid, 5);
    generate_cannonballs(&mut grid);

    print_grid(&grid);
}
