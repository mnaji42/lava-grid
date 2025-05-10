#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_game_over() {
        assert_eq!(check_game_over(), false);
    }
}