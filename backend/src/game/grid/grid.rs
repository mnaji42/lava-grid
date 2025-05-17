use crate::game::types::{Cell, Position, GameMode};
use crate::game::state::GameState;
use rand::seq::{IteratorRandom};

pub fn generate_grid(rows: usize, cols: usize) -> Vec<Vec<Cell>> {
    vec![vec![Cell::Solid; cols]; rows]
}

fn is_cell_breakable(cell: Cell) -> bool {
    cell == Cell::Solid
}

pub fn break_tile(game_state: &mut GameState) {
    let mut rng = rand::rng();

    match game_state.mode {
        GameMode::Classic => {
            let solid_tiles: Vec<(usize, usize)> = game_state.grid.iter().enumerate()
                .flat_map(|(y, row)| row.iter().enumerate().filter_map(move |(x, cell)| {
                    if *cell == Cell::Solid {
                        Some((y, x))
                    } else {
                        None
                    }
                }))
                .collect();

            if let Some(&(y, x)) = solid_tiles.iter().choose(&mut rng) {
                if let Some(pos) = game_state.cannonballs.iter().position(|c| c.pos == Position { x, y }) {
                    game_state.cannonballs.remove(pos);
                }
                game_state.grid[y][x] = Cell::Broken;
            }
        }
        GameMode::Cracked => {
            // 1. Toutes les Cracked deviennent Broken
            for row in game_state.grid.iter_mut() {
                for cell in row.iter_mut() {
                    if *cell == Cell::Cracked {
                        *cell = Cell::Broken;
                    }
                }
            }
            // 2. On choisit une Solid à passer en Cracked
            let solid_tiles: Vec<(usize, usize)> = game_state.grid.iter().enumerate()
                .flat_map(|(y, row)| row.iter().enumerate().filter_map(move |(x, cell)| {
                    if *cell == Cell::Solid {
                        Some((y, x))
                    } else {
                        None
                    }
                }))
                .collect();

            if let Some(&(y, x)) = solid_tiles.iter().choose(&mut rng) {
                if let Some(pos) = game_state.cannonballs.iter().position(|c| c.pos == Position { x, y }) {
                    game_state.cannonballs.remove(pos);
                }
                game_state.grid[y][x] = Cell::Cracked;
            }
        }
    }
}