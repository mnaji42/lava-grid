use crate::game::systems::{move_player, apply_rules, print_player_state, print_grid};
use crate::game::types::{Direction};
use crate::game::state::GameState;

use std::io::{self, Write};

fn get_player_input() -> Direction {
    print!("Enter direction (← ↑ ↓ → or Space), then press Enter: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    match input.trim() {
        "\x1b[D" => Direction::Left,
        "\x1b[C" => Direction::Right,
        "\x1b[A" => Direction::Up,
        "\x1b[B" => Direction::Down,
        _ => Direction::Stay,
    }
}

pub fn run_game_loop() {
    let player_id = 0;
    let mut game_state = GameState::new(5, 5, 1); // Initialisation de l'état du jeu

    println!("Game start!");
    print_player_state(&game_state.players[0]);
    print_grid(&game_state.grid, &game_state.players, &game_state.cannonballs);

    loop {
        let direction = get_player_input();
        move_player(&mut game_state, player_id, direction);
        
        apply_rules(&mut game_state, player_id);

        game_state.next_turn(); // Passer au tour suivant

        print_player_state(&game_state.players[0]);
        print_grid(&game_state.grid, &game_state.players, &game_state.cannonballs);

        if !game_state.players[0].is_alive {
            println!("Player {} is dead. Game Over!", game_state.players[0].id);
            break;
        }
    }
}