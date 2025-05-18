# WebSocket API Documentation

## Overview

This document describes all WebSocket messages that the backend server can send to clients in both the **matchmaking lobby** and **game session** contexts.  
It is intended for frontend developers and integrators who need to interact with the backend via WebSocket.

All messages are sent as JSON objects with the following structure:

```json
{
  "action": "<ActionName>",
  "data": { ... }
}
```

- `action`: A string identifying the type of message.
- `data`: An object containing the message payload (may be omitted for some actions).

---

## Table of Contents

- [General Conventions](#general-conventions)
- [Matchmaking WebSocket Messages](#matchmaking-websocket-messages)
  - [UpdateState](#updatestate)
  - [GameStarted](#gamestarted)
  - [Error](#error)
  - [SessionKicked](#sessionkicked)
- [Game Session WebSocket Messages](#game-session-websocket-messages)
  - [GameInit](#gameinit)
  - [GameStateUpdate](#gamestateupdate)
  - [GameEnded](#gameended)
  - [Error](#error-1)
  - [SessionKicked](#sessionkicked-1)
  - [GamePreGameData](#gamepregamedata)
  - [GameModeVoteUpdate](#gamemodevoteupdate)
  - [GameModeChosen](#gamemodechosen)
  - [CustomMessage](#custommessage)
- [Error Codes Reference](#error-codes-reference)
- [Examples](#examples)

---

## General Conventions

- All messages are UTF-8 encoded JSON.
- The `action` field is always present and determines the message type.
- The `data` field contains the payload, which varies by message.
- Error messages always include a `code`, `message`, and optional `context`.

---

## Matchmaking WebSocket Messages

These messages are sent on the `/ws/matchmaking` WebSocket endpoint.

### `UpdateState`

**Purpose:**  
Sent to all clients whenever the state of the matchmaking lobby changes (players join/leave, ready status, countdown, etc).

**Format:**

```json
{
  "action": "UpdateState",
  "data": {
    "lobby_players": [PlayerInfo],
    "ready_players": [PlayerInfo],
    "countdown_active": true|false,
    "countdown_remaining": 12
  }
}
```

**Fields:**

- `lobby_players`: Array of players currently in the lobby (not ready).
- `ready_players`: Array of players who have paid and are ready to play.
- `countdown_active`: Boolean, true if a countdown to game start is active.
- `countdown_remaining`: Number of seconds remaining in the countdown (if active), or null.

**PlayerInfo structure:**

```json
{
  "id": "wallet_address",
  "username": "display_name"
}
```

---

### `GameStarted`

**Purpose:**  
Notifies the client that a new game has started and provides the assigned game ID.

**Format:**

```json
{
  "action": "GameStarted",
  "data": {
    "game_id": "uuid-string"
  }
}
```

**Fields:**

- `game_id`: UUID string identifying the new game session.

---

### `Error`

**Purpose:**  
Notifies the client of an error (invalid action, protocol error, etc).

**Format:**

```json
{
  "action": "Error",
  "data": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "context": "optional context string"
  }
}
```

**Fields:**

- `code`: Unique error code (see [Error Codes Reference](#error-codes-reference)).
- `message`: Human-readable description of the error.
- `context`: Optional string providing additional context (e.g., wallet address).

---

### `SessionKicked`

**Purpose:**  
Notifies the client that their session has been disconnected because another session has connected with the same wallet.

**Format:**

```json
{
  "action": "SessionKicked",
  "data": {
    "reason": "Explanation for the kick"
  }
}
```

**Fields:**

- `reason`: String explaining why the session was kicked.

---

## Game Session WebSocket Messages

These messages are sent on the `/ws/game/{game_id}` WebSocket endpoint.

### `GameInit`

**Purpose:**  
Sent at the start of the game, providing the initial game state and chosen mode.

**Format:**

```json
{
  "action": "GameInit",
  "data": {
    "state": { ... },      // See GameState structure
    "mode": "Classic"      // or "Cracked"
  }
}
```

**Fields:**

- `state`: The initial game state (see below).
- `mode`: The chosen game mode.

---

### `GameStateUpdate`

**Purpose:**  
Sent after each turn, providing the updated game state and turn duration.

**Format:**

```json
{
  "action": "GameStateUpdate",
  "data": {
    "state": { ... },          // See GameState structure
    "turn_duration": 20        // Duration in seconds for the next turn
  }
}
```

**Fields:**

- `state`: The updated game state.
- `turn_duration`: Number of seconds for the next turn.

---

### `GameEnded`

**Purpose:**  
Notifies clients that the game has ended and provides the winner.

**Format:**

```json
{
  "action": "GameEnded",
  "data": {
    "winner": "username"
  }
}
```

**Fields:**

- `winner`: Username of the winning player.

---

### `Error`

**Purpose:**  
Notifies the client of an error (invalid action, protocol error, etc).

**Format:**  
Same as in [Matchmaking Error](#error).

---

### `SessionKicked`

**Purpose:**  
Notifies the client that their session has been disconnected because another session has connected with the same wallet for this game.

**Format:**  
Same as in [Matchmaking SessionKicked](#sessionkicked).

---

### `GamePreGameData`

**Purpose:**  
Sent at the start of the pre-game phase (mode choice), providing available modes, deadline, player list, and grid size.

**Format:**

```json
{
  "action": "GamePreGameData",
  "data": {
    "modes": ["Classic", "Cracked"],
    "deadline_secs": 30,
    "players": [PlayerInfo],
    "grid_row": 10,
    "grid_col": 10
  }
}
```

**Fields:**

- `modes`: Array of available game modes.
- `deadline_secs`: Number of seconds until mode choice deadline.
- `players`: Array of participating players (see PlayerInfo).
- `grid_row`: Number of rows in the game grid.
- `grid_col`: Number of columns in the game grid.

---

### `GameModeVoteUpdate`

**Purpose:**  
Notifies all clients when a player votes for a game mode.

**Format:**

```json
{
  "action": "GameModeVoteUpdate",
  "data": {
    "player_id": "wallet_address",
    "mode": "Classic" // or "Cracked"
  }
}
```

**Fields:**

- `player_id`: Wallet address of the player who voted.
- `mode`: The mode they voted for.

---

### `GameModeChosen`

**Purpose:**  
Notifies all clients of the chosen game mode and which player was selected to decide.

**Format:**

```json
{
  "action": "GameModeChosen",
  "data": {
    "mode": "Classic", // or "Cracked"
    "chosen_by": "wallet_address"
  }
}
```

**Fields:**

- `mode`: The chosen game mode.
- `chosen_by`: Wallet address of the player whose vote was selected (or who was randomly chosen).

---

### `CustomMessage`

**Purpose:**  
Sends a custom text message to the client (for notifications or debugging).

**Format:**

```json
{
  "action": "CustomMessage",
  "data": {
    "text": "Arbitrary message"
  }
}
```

**Fields:**

- `text`: The message content.

---

## Error Codes Reference

Below are common error codes that may be sent in `Error` messages:

| Code                    | Context          | Description                                               |
| ----------------------- | ---------------- | --------------------------------------------------------- |
| `INVALID_ACTION`        | Matchmaking/Game | The client sent an invalid or malformed command.          |
| `WS_PROTOCOL_ERROR`     | Matchmaking/Game | WebSocket protocol error.                                 |
| `SERIALIZATION_ERROR`   | Matchmaking/Game | Internal server error serializing a message.              |
| `BANNED`                | Matchmaking/Game | The client has been banned for spamming.                  |
| `SESSION_KICKED`        | Matchmaking/Game | Another session connected with the same wallet.           |
| `MISSING_WALLET`        | Matchmaking/Game | The wallet address was missing in the connection request. |
| `INVALID_GAME_ID`       | Game             | The provided game_id is invalid.                          |
| `GAME_SESSION_ERROR`    | Game             | Internal error creating or finding the game session.      |
| `MAILBOX_ERROR`         | Game             | Internal error communicating with the actor system.       |
| `GAME_NOT_STARTED`      | Game             | The game has not started yet.                             |
| `TURN_NOT_IN_PROGRESS`  | Game             | The turn is not currently in progress.                    |
| `UNKNOWN_PLAYER`        | Game             | The client is not recognized as a player in this game.    |
| `PLAYER_ELIMINATED`     | Game             | The player is eliminated and cannot act.                  |
| `ALREADY_ACTED`         | Game             | The player has already acted this turn.                   |
| `SPECTATOR_COMMAND`     | Game             | Spectators cannot send commands.                          |
| `SESSION_ADDR_MISMATCH` | Game             | The session address does not match the registered one.    |

> **Note:** Additional error codes may be added as the backend evolves.

---

## Examples

### Example: Matchmaking State Update

```json
{
  "action": "UpdateState",
  "data": {
    "lobby_players": [{ "id": "0x123...", "username": "Alice" }],
    "ready_players": [{ "id": "0x456...", "username": "Bob" }],
    "countdown_active": true,
    "countdown_remaining": 15
  }
}
```

### Example: Game State Update

```json
{
  "action": "GameStateUpdate",
  "data": {
    "state": {
      "turn": 3,
      "players": [
        {
          "id": "0x123...",
          "username": "Alice",
          "pos": [1, 2],
          "is_alive": true
        }
      ]
      // ... other game state fields
    },
    "turn_duration": 20
  }
}
```

### Example: Error

```json
{
  "action": "Error",
  "data": {
    "code": "INVALID_ACTION",
    "message": "Invalid command",
    "context": "0x123..."
  }
}
```

### Example: Session Kicked

```json
{
  "action": "SessionKicked",
  "data": {
    "reason": "Another session has connected with your wallet."
  }
}
```

---

## Appendix: GameState Structure

The `state` object in game messages is defined by the backend and may include fields such as:

- `turn`: Current turn number.
- `players`: Array of player objects (id, username, position, alive status, etc).
- Additional fields depending on game mode.

Consult the backend code or ask the backend team for the full schema.

---

## Contact

For questions or to report inconsistencies in this documentation, please contact the backend team.
