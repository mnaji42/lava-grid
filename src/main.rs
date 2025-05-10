mod rules;
mod rules_dev;

fn main() {
    let grid = rules::generate_grid(10, 10);
    rules_dev::print_grid(&grid);
}