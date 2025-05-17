//! Core types and enums for the game logic.
//!
//! This module defines the main data structures used throughout the game engine,
//! including player, grid, movement, and game mode representations.

use serde::{Serialize, Deserialize};

/// Available game modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameMode {
    /// Classic mode: standard rules.
    Classic,
    /// Cracked mode: special rules (see game logic).
    Cracked,
}

/// Position on the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    /// X coordinate (column).
    pub x: usize,
    /// Y coordinate (row).
    pub y: usize,
}

/// Possible movement directions for a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    /// No movement.
    Stay,
}

/// State of a cell on the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Cell {
    /// Solid tile (can be walked on).
    Solid,
    /// Cracked tile (may break soon).
    Cracked,
    /// Broken tile (cannot be walked on, lethal).
    Broken,
}

/// Player representation in the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// Unique player ID (assigned by order of joining).
    pub id: u8,
    /// Display username.
    pub username: String,
    /// Current position on the grid.
    pub pos: Position,
    /// Number of cannonballs the player holds.
    pub cannonball_count: u32,
    /// Whether the player is alive.
    pub is_alive: bool,
}

impl Player {
    /// Create a new player at a given position.
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

/// A cannonball present on the grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cannonball {
    /// Position of the cannonball.
    pub pos: Position,
}

/// A tile targeted by a cannonball shot (to be broken at end of turn).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetedTile {
    /// X coordinate of the targeted tile.
    pub x: usize,
    /// Y coordinate of the targeted tile.
    pub y: usize,
}