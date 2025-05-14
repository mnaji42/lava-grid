use crate::game::types::{Player, Cell, Cannonball};

// pub fn build_display_grid(
//     grid: &Vec<Vec<Cell>>,
//     players: &Vec<Player>,
//     cannonballs: &Vec<Cannonball>,
// ) -> Vec<Vec<char>> {
//     let mut display = grid.iter().map(|row| {
//         row.iter().map(|cell| match cell {
//             Cell::Solid => '.',
//             Cell::Broken => '~',
//         }).collect::<Vec<_>>()
//     }).collect::<Vec<_>>();

//     for cannon in cannonballs {
//         display[cannon.pos.x][cannon.pos.y] = 'B';
//     }

//     for player in players {
//         if player.is_alive {
//             display[player.pos.x][player.pos.y] = char::from_digit(player.id as u32, 10).unwrap_or('P');
//         }
//     }

//     display
// }

pub fn print_grid(grid: &Vec<Vec<Cell>>, players: &Vec<Player>, cannonballs: &Vec<Cannonball>) {
    for (y, row) in grid.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            let mut symbol = match cell {
                Cell::Solid => "██".to_string(),
                Cell::Broken => "  ".to_string(),
            };

            // Priorité à l'affichage du joueur puis cannonball
            if let Some(player) = players.iter().find(|p| p.pos.x == x && p.pos.y == y && p.is_alive) {
                symbol = format!("P{}", player.id);
            } else if let Some(_) = cannonballs.iter().find(|c| c.pos.x == x && c.pos.y == y) {
                symbol = "CB".to_string();
            }

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