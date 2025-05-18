// src/server/mod.rs

//! Server layer root module.
//!
//! This module organizes the main backend server components, including:
//! - Application state management
//! - HTTP/WebSocket routing
//! - Matchmaking logic (lobby, payments, player readiness)
//! - Game session orchestration (game lifecycle, player actions)

pub mod state;
pub mod router;
pub mod matchmaking;
pub mod game_session;
pub mod ws_error;