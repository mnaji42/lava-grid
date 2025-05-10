pub fn print_grid(grid: &Vec<Vec<u8>>) {
    for row in grid {
        println!("{:?}", row);
    }
}

pub fn debug_print(message: &str) {
    println!("DEBUG: {}", message);
}