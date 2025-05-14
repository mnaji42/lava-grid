use crate::game::types::{Cell, Position};
use crate::game::state::GameState;
use rand::{seq::IteratorRandom};

pub fn generate_grid(rows: usize, cols: usize) -> Vec<Vec<Cell>> {
    vec![vec![Cell::Solid; cols]; rows]
}

fn is_cell_breakable(cell: Cell) -> bool {
    cell == Cell::Solid
}


// pub fn break_tile(grid: &mut Vec<Vec<Cell>>, cannonballs: &mut Vec<Cannonball>) {
pub fn break_tile(game_state: &mut GameState) {
    let mut rng = rand::rng();
    
    // Filtrer les cases solides
    let solid_tiles: Vec<(usize, usize)> = game_state.grid.iter().enumerate()
        .flat_map(|(x, row)| row.iter().enumerate().filter_map(move |(y, cell)| {
            if *cell == Cell::Solid {
                Some((x, y))
            } else {
                None
            }
        }))
        .collect();

    if let Some(&(x, y)) = solid_tiles.iter().choose(&mut rng) {
        // Chercher s'il y a un canon sur cette tuile et le supprimer
        if let Some(pos) = game_state.cannonballs.iter().position(|c| c.pos == Position { x, y }) {
            game_state.cannonballs.remove(pos);  // Supprimer le canon
        }

        // Casser la tuile
        game_state.grid[y][x] = Cell::Broken;
    }
}



