use rand::{seq::IteratorRandom, thread_rng, Rng};

#[derive(Debug, Clone, Copy)]
pub enum Cell {
    Solid,        // Solid cell
    Broken,       // Broken cell (lava)
    Player(u8),   // Player with their ID
    Cannonball,   // Cannonball
}

impl Cell {
    // Returns true if the cell is solid
    pub fn is_solid(&self) -> bool {
        matches!(self, Cell::Solid)
    }

    // Returns true if the cell is broken (lava)
    pub fn is_broken(&self) -> bool {
        matches!(self, Cell::Broken)
    }
}

#[derive(Clone, Debug)]
pub struct Cannonball {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: u8,
    pub x: usize,
    pub y: usize,
    pub has_cannonball: bool,
}

// Generates a grid of size (rows x cols), filled with '1' (solid)
pub fn generate_grid(rows: usize, cols: usize) -> Vec<Vec<Cell>> {
    // Fill the grid with solid cells
    vec![vec![Cell::Solid; cols]; rows]
}

pub fn initialize_players(grid: &mut Vec<Vec<Cell>>, num_players: usize) -> Vec<Player> {
    let mut players = Vec::new();
    let mut rng = thread_rng();

    // Collect all available solid positions in the grid
    let mut available_positions: Vec<(usize, usize)> = grid.iter()
        .enumerate()
        .flat_map(|(x, row)| {
            row.iter().enumerate()
                .filter_map(move |(y, cell)| if matches!(cell, Cell::Solid) { Some((x, y)) } else { None })
        })
        .collect();

    // Randomly assign a position to each player
    for id in 1..=num_players {
        if let Some(&(x, y)) = available_positions.iter().choose(&mut rng) {
            grid[x][y] = Cell::Player(id as u8);
            players.push(Player {
                id: id as u8,
                x,
                y,
                has_cannonball: false,
            });

            // Remove the used position to avoid duplicates
            available_positions.retain(|&(a, b)| !(a == x && b == y));
        }
    }

    players
}

// Function to place cannonballs randomly on the grid
pub fn generate_cannonballs(grid: &mut Vec<Vec<Cell>>) {
    let mut rng = thread_rng();
    
    // Choose a random number of cannonballs to place (between 1 and 3)
    let num_cannonballs = rand::thread_rng().gen_range(1..=3);
    
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