 use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Stay,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Cell {
    Solid,
    Broken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: u8,
    pub username: String,
    pub pos: Position,
    pub cannonball_count: u32,
    pub is_alive: bool,
}

impl Player {
    pub fn new(id: u8, pos: Position, username: String) -> Self {
        Self {
            id,
            pos,
            username,
            cannonball_count: 0,
            is_alive: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cannonball {
    pub pos: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetedTile {
    pub x: usize,
    pub y: usize,
}
