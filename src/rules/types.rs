/////////////////////////////////////////////
// ************* GRID *********************//

#[derive(Debug, Clone, Copy)]
pub enum Cell {
    Solid,        // Solid cell
    Broken,       // Broken cell (lava)
    Player(u8),   // Player with their ID
    Cannonball,   // Cannonball
}

impl Cell {
    pub fn is_solid(&self) -> bool {
        matches!(self, Cell::Solid)
    }

    pub fn is_broken(&self) -> bool {
        matches!(self, Cell::Broken)
    }
}

/////////////////////////////////////////////
// ************* Player *********************//

#[derive(Debug, Clone)]
pub struct Player {
    pub id: u8,
    pub x: usize,
    pub y: usize,
    pub has_cannonball: bool,
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Stay,
}

/////////////////////////////////////////////
// ************* CANONBALL *********************//

#[derive(Clone, Debug)]
pub struct Cannonball {
    pub x: usize,
    pub y: usize,
}
