# Commenting & Documentation Guidelines

This document describes the conventions for code comments and documentation in the Lava Grid project.  
**Please read and follow these guidelines when contributing!**

---

## 📦 Table of Contents

- [General Principles](#general-principles)
- [Rust (Backend)](#rust-backend)
- [JavaScript/TypeScript (Frontend)](#javascripttypescript-frontend)
- [Best Practices](#best-practices)
- [AI & Open Source Tips](#ai--open-source-tips)
- [Where to Start](#where-to-start)

---

## General Principles

- **Language:** All comments and documentation must be in English.
- **Clarity:** Write for someone who does not know the codebase. Be explicit about business logic and technical details.
- **Minimalism:** Only comment where necessary—avoid redundant or excessive comments.
- **Audience:** Comments should help:
  - Yourself (future you!),
  - Other developers (open source),
  - AI tools (for code understanding and generation).
- **Consistency:** Use the same style everywhere.

---

## Rust (Backend)

_These conventions apply to all Rust code in the backend._

### 1. Types of Comments

#### 1.1. Rust Doc Comments (`///`)

- Use triple slashes `///` for documenting public items (modules, structs, enums, functions, traits, constants, etc.).
- These comments are used by `cargo doc` to generate documentation.
- Always start doc comments with a summary line, followed by a blank line, then details/examples if needed.
- **Do not over-comment:** Only document what is not obvious from the code.

**Example:**

```rust
/// Represents a player in the game.
///
/// Each player has a unique ID, a username, a position on the grid,
/// a number of available cannonballs, and a status indicating if they are alive.
pub struct Player {
    pub id: u8,
    pub username: String,
    pub pos: Position,
    pub cannonball_count: u32,
    pub is_alive: bool,
}
```

#### 1.2. Inline Comments (`//`)

- Use `//` for short explanations inside function bodies or for clarifying complex logic.
- Place them above the line they explain, or at the end if very short.
- **Avoid commenting obvious code.**

**Example:**

```rust
// Remove the player from the ready group if present
if let Some(player) = group.remove(player_id) {
    return Some(player);
}
```

#### 1.3. Block Comments (`/* ... */`)

- Rarely used. Only for temporarily commenting out code or for large explanations that don’t fit doc comments.

---

### 2. What to Document

- **Modules:** At the top of each module, add a doc comment explaining its purpose and responsibilities.
- **Structs, Enums, Traits:** Describe what the type represents and its role in the business logic.
- **Functions & Methods:** Always document public functions/methods. Use summary, arguments, returns, errors, and examples if needed.
- **Constants:** Explain what the constant is for, and why the value was chosen if not obvious.

---

### 3. Template for Doc Comments

````rust
/// [Short summary of what this item does or represents]
///
/// [Longer explanation, business logic, and technical details]
///
/// # Arguments
/// * `arg1` - [Description]
/// * `arg2` - [Description]
///
/// # Returns
/// [Description of return value]
///
/// # Errors
/// [Possible errors]
///
/// # Example
/// ```
/// // Example usage
/// ```
````

---

## JavaScript/TypeScript (Frontend)

_These conventions apply to all JS/TS code in the frontend (Next.js, React, etc.)._

### 1. Types of Comments

#### 1.1. JSDoc Comments (`/** ... */`)

- Use JSDoc-style comments for documenting functions, classes, components, and modules.
- Start with a summary line, then describe parameters, return values, and examples if needed.
- **Do not over-comment:** Only document what is not obvious from the code.

**Example:**

```typescript
/**
 * Renders the player avatar.
 *
 * @param {string} username - The player's username.
 * @param {string} avatarUrl - URL to the player's avatar image.
 * @returns {JSX.Element} The rendered avatar component.
 *
 * @example
 * <PlayerAvatar username="Alice" avatarUrl="/avatars/alice.png" />
 */
function PlayerAvatar({ username, avatarUrl }: Props) {
  // ...
}
```

#### 1.2. Inline Comments (`//`)

- Use `//` for short explanations inside functions or to clarify complex logic.
- **Avoid commenting obvious code.**

**Example:**

```typescript
// Filter out inactive players before rendering
const activePlayers = players.filter((p) => p.isActive)
```

#### 1.3. Block Comments (`/* ... */`)

- Use sparingly, for temporarily commenting out code or for large explanations.

---

### 2. What to Document

- **Modules/Files:** At the top, describe the purpose of the file/module.
- **Functions/Components:** Document all exported/public functions and React components.
- **Parameters & Returns:** Use `@param` and `@returns` in JSDoc.
- **Props:** For React components, document the props interface/type.
- **Constants:** Explain non-obvious constants.

---

### 3. Template for JSDoc Comments

```typescript
/**
 * [Short summary of what this function/component does]
 *
 * [Longer explanation, business logic, and technical details]
 *
 * @param {Type} paramName - [Description]
 * @returns {Type} [Description of return value]
 *
 * @example
 * // Example usage
 */
```

---

## Best Practices

- **Be concise but complete:** Don’t repeat what the code says, but explain the “why” and the business logic.
- **Update comments:** Always update documentation when changing code.
- **Use examples:** For complex functions, add usage examples in doc comments.
- **Document edge cases and errors:** Especially for public APIs.
- **Minimalism:** When in doubt, prefer fewer comments, but ensure all business logic and non-obvious decisions are explained.

---

## AI & Open Source Tips

- **Explicit business logic:** Clearly explain game rules, state transitions, and protocol details.
- **Describe message formats:** For WebSocket messages, document the structure and meaning.
- **Cross-reference:** Use links or references to related modules/types when helpful.

---

## Where to Start

- For Rust: Begin with the configuration files (`src/config/`).
- For JS/TS: Begin with top-level components and utility functions.
- For each file, add module-level doc comments, then document each public item.
- Use these conventions for all new code and when refactoring.

---

**Feel free to adapt and extend these conventions as your project evolves!**
