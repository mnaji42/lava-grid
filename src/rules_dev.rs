use crate::rules::types::{Cell}; 

pub fn print_grid(grid: &Vec<Vec<Cell>>) {
    for row in grid {
        println!("{:?}", row);
    }
}

pub fn debug_print(message: &str) {
    println!("DEBUG: {}", message);
}