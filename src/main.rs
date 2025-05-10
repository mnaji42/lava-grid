mod rules;
mod rules_dev;
use crate::rules::{Cell, generate_grid, initialize_players, generate_cannonballs};
use crate::rules_dev::{print_grid, debug_print};


fn main() {
    let mut grid = generate_grid(10, 10); // Generate a 10x10 grid
    let players = initialize_players(&mut grid, 5); // Initialize 5 players on the grid
    generate_cannonballs(&mut grid);

    print_grid(&grid);

    // // Display the players (for debugging purposes)
    // for player in players {
    //     println!("{:?}", player); // Print the player's state
    // }
}
