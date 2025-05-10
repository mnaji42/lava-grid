use crate::rules::{
    Player, Cell, Direction,
    move_player, get_player_move, break_tile
};
use crate::rules_dev::{print_grid,print_player_state};
use std::io::{self, Write};

/// Run the game loop for a fixed number of turns
pub fn run_game_loop(grid: &mut Vec<Vec<Cell>>, players: &mut Vec<Player>, turns: usize) {
    println!("hello world... game start");
    loop {
        print!("Enter direction (← ↑ ↓ → or Space), then press Enter: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let dir = match input.trim() {
            "\x1b[D" => Direction::Left,
            "\x1b[C" => Direction::Right,
            "\x1b[A" => Direction::Up,
            "\x1b[B" => Direction::Down,
            _ => Direction::Stay,
        };
        move_player(&mut players[0], dir, grid);
        break_tile(grid);
        print_player_state(&players[0]);
        print_grid(grid);
        if !players[0].is_alive {
            println!("Player {} is dead. Game Over!", players[0].id);
            break;
        }
    }
}
