/////////////////////////////////////////////
// ************* GENERAL *****************//

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

/////////////////////////////////////////////
// ************* GRID *********************//

#[derive(Debug, Clone, Copy, PartialEq)]
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
    pub pos: Position,
    pub cannonball_count: u32,
    pub is_alive: bool,
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
    pub pos: Position
}
