mod rules;
mod rules_dev;

use crate::rules::{generate_grid, initialize_players, generate_cannonballs};
use crate::rules::game::run_game_loop;
use crate::rules_dev::print_grid;

fn main() {
    let mut grid = generate_grid(5, 5);
    let mut players = initialize_players(&mut grid, 1);
    generate_cannonballs(&mut grid);

    println!("=== Initial Grid ===");
    print_grid(&grid);

    // Run the game loop for 5 turns
    run_game_loop(&mut grid, &mut players, 5);
}
