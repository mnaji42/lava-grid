mod game;
mod types;
mod utils;
mod state;

mod entities {
    pub mod player;
    pub mod cannonball;
}

mod grid {
    pub mod grid;
}

mod systems {
    pub mod movement;
    pub mod rules;
    pub mod render;
}

use crate::game::run_game_loop;

fn main() {
    run_game_loop();
}