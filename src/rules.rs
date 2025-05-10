use rand::Rng;

pub fn check_game_over() -> bool {
    // Logique pour déterminer si le jeu est terminé
    false
}

// Grid 1 = solid and Grid 0 broken (lava)
pub fn generate_grid(rows: usize, cols: usize) -> Vec<Vec<u8>> {
    vec![vec![1; cols]; rows]
}
