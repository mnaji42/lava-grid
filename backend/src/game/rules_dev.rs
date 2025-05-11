use self::types::{Cell,Player}; 

pub fn print_grid(grid: &Vec<Vec<Cell>>) {
    for row in grid {
        for cell in row {
            let symbol = match cell {
                Cell::Solid => "██".to_string(),
                Cell::Broken => "  ".to_string(),
                Cell::Cannonball => "CB".to_string(),
                Cell::Player(id) => format!("P{id}"),
            };

            print!("{:<3}", symbol);
        }
        println!("\n");
    }
}

pub fn print_player_state(player: &Player) {
    println!("--- Player {} ---", player.id);
    println!("Position: ({}, {})", player.pos.x, player.pos.y);
    println!("Cannonballs: {}", player.cannonball_count);
    println!();
}

pub fn debug_print(message: &str) {
    println!("DEBUG: {}", message);
}