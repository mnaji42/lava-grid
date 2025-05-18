/// Handles turn start and resolution logic for GameSession.
/// Encapsulates timer management, action collection, and state updates.

use std::time::{Duration, Instant};
use actix::prelude::*;

use crate::server::game_session::server::GameSession;
use crate::server::game_session::messages::GameStateUpdate;
use crate::config::game::TURN_DURATION;

/// Start a new turn: reset actions, launch timer, broadcast state.
pub fn start_new_turn(this: &mut GameSession, ctx: &mut Context<GameSession>) {
    if this.game_state.is_none() {
        return;
    }
    this.turn_in_progress = true;
    this.pending_actions.clear();
    this.turn_start_time = Some(Instant::now());

    // Start the turn timer.
    let handle = ctx.run_later(Duration::from_secs(TURN_DURATION), |act, ctx| {
        resolve_turn(act, ctx);
    });
    this.turn_timer = Some(handle);

    // Broadcast the new turn state.
    if let Some(ref state) = this.game_state {
        let turn_duration = this.get_turn_remaining_secs();
        for addr in this.players.values().chain(this.spectators.values()) {
            addr.do_send(GameStateUpdate { state: state.clone(), turn_duration });
        }
    }
}

/// Resolve the current turn: apply actions, update state, check for game end.
pub fn resolve_turn(this: &mut GameSession, ctx: &mut Context<GameSession>) {
    if this.game_state.is_none() || !this.turn_in_progress {
        return;
    }
    this.turn_in_progress = false;

    let state = this.game_state.as_mut().unwrap();

    // For each living player, if no action was received, default to Stay.
    for info in &this.player_infos {
        if !this.pending_actions.contains_key(&info.id) {
            if let Some(_idx) = state.players.iter().position(|p| p.username == info.username && p.is_alive) {
                this.pending_actions.insert(info.id.clone(), crate::server::game_session::messages::PlayerAction::Move(crate::game::types::Direction::Stay));
            }
        }
    }

    // Apply all actions in player order.
    for (i, info) in this.player_infos.iter().enumerate() {
        if let Some(player) = state.players.get(i) {
            if !player.is_alive { continue; }
        }
        if let Some(action) = this.pending_actions.get(&info.id) {
            state.apply_player_action(action.clone(), i);
        }
    }

    // Advance the turn counter.
    state.next_turn();

    // If more than one player is alive, start the next turn.
    if state.players.iter().filter(|p| p.is_alive).count() > 1 {
        start_new_turn(this, ctx);
    } else {
        // Game is over, notify all clients.
        for addr in this.players.values().chain(this.spectators.values()) {
            addr.do_send(GameStateUpdate { state: state.clone(), turn_duration: 0 });
            // TODO: send a GameEnded message if needed
        }
    }
}
