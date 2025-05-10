use crate::types::{Cell};
use rand::{seq::IteratorRandom};

pub fn generate_grid(rows: usize, cols: usize) -> Vec<Vec<Cell>> {
    vec![vec![Cell::Solid; cols]; rows]
}

fn is_cell_breakable(cell: Cell) -> bool {
    cell == Cell::Solid
}


pub fn break_tile(grid: &mut Vec<Vec<Cell>>) {
    let mut rng = rand::rng();
    let solid_tiles: Vec<(usize, usize)> = grid.iter().enumerate()
        .flat_map(|(x, row)| row.iter().enumerate().filter_map(move |(y, cell)| {
            if is_cell_breakable(*cell) {
                Some((x, y))
            } else {
                None
            }
        }))
        .collect();

    if let Some(&(x, y)) = solid_tiles.iter().choose(&mut rng) {
        grid[x][y] = Cell::Broken;
    }
}


