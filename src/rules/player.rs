use rand::{seq::IteratorRandom, rngs::ThreadRng};
use crate::rules::types::{Player, Direction, Cell, Position}; 

pub fn initialize_players(grid: &mut Vec<Vec<Cell>>, num_players: usize) -> Vec<Player> {
    let mut players = Vec::new();
    let mut rng: ThreadRng = rand::thread_rng();

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
                pos: Position { x, y },
                is_alive: true,
                cannonball_count: 0,
            });

            // Remove the used position to avoid duplicates
            available_positions.retain(|&(a, b)| !(a == x && b == y));
        }
    }

    players
}

// Function to get the player's move (this is a placeholder for now)
pub fn get_player_move() -> Direction {
    // Simulate the player making a choice (you can replace this with actual input later)
    Direction::Stay // For now, we just return Stay
}

// Function to move the player
pub fn move_player(player: &mut Player, direction: Direction, grid: &mut Vec<Vec<Cell>>) {
    let mut new_pos = player.pos;

    match direction {
        Direction::Up => {
            if player.pos.x > 0 {
                new_pos.x -= 1;
            }
        }
        Direction::Down => {
            if player.pos.x < grid.len() - 1 {
                new_pos.x += 1;
            }
        }
        Direction::Left => {
            if player.pos.y > 0 {
                new_pos.y -= 1;
            }
        }
        Direction::Right => {
            if player.pos.y < grid[0].len() - 1 {
                new_pos.y += 1;
            }
        }
        Direction::Stay => {
            // Player stays in the same position
        }
    }

    if grid[new_pos.x][new_pos.y] == Cell::Cannonball {
        player.cannonball_count += 1;
    }
    grid[player.pos.x][player.pos.y] = Cell::Solid;
    player.pos = new_pos;
    if grid[new_pos.x][new_pos.y] == Cell::Broken 
        {player.is_alive = false;}
    else 
        {grid[player.pos.x][player.pos.y] = Cell::Player(player.id);}
}