mod rules;

fn main() {
    if rules::check_game_over() {
        println!("Game Over!");
    }
}