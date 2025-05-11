#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Stay,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Solid,
    Broken,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: u8,
    pub pos: Position,
    pub cannonball_count: u32,
    pub is_alive: bool,
}

impl Player {
    pub fn new(id: u8, pos: Position) -> Self {
        Self {
            id,
            pos,
            cannonball_count: 0,
            is_alive: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cannonball {
    pub pos: Position,
}

#[derive(Debug, Clone)]
pub struct TargetedTile {
    pub x: usize,
    pub y: usize,
}