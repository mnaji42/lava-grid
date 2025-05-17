//! Game rendering system (terminal).
//!
//! This module provides functions to print the grid and player state for debugging/demo.

use crate::game::types::{Player, Cell, Cannonball};

/// Print the grid, players, and cannonballs to the terminal.
pub fn print_grid(grid: &Vec<Vec<Cell>>, players: &Vec<Player>, cannonballs: &Vec<Cannonball>) {
    for (y, row) in grid.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            // Default symbol for the cell.
            let mut symbol = match cell {
                Cell::Solid => "██".to_string(),
                Cell::Cracked => "Ck".to_string(),
                Cell::Broken => "  ".to_string(),
            };

            // If a player is alive on this tile, display the player.
            if let Some(player) = players.iter().find(|p| p.pos.x == x && p.pos.y == y && p.is_alive) {
                symbol = format!("P{}", player.id);
            }
            // If no player, but a cannonball is present, display the cannonball.
            else if let Some(_) = cannonballs.iter().find(|c| c.pos.x == x && c.pos.y == y) {
                symbol = "CB".to_string();
            }

            print!("{:<3}", symbol);
        }
        println!("\n");
    }
}

/// Print the state of a single player.
pub fn print_player_state(player: &Player) {
    println!("--- Player {} ---", player.id);
    println!("Position: ({}, {})", player.pos.x, player.pos.y);
    println!("Cannonballs: {}", player.cannonball_count);
    println!();
}