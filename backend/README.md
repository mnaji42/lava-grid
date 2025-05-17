# Lava Grid — Backend

**Rust backend for Lava Grid: real-time, trustless, ZK-powered strategy game.**

---

## 🏗️ Architecture Overview

The backend is organized in three main layers:

- **Configuration (`src/config/`)**: Game and matchmaking constants.
- **Server Layer (`src/server/`)**: Networking, matchmaking, game sessions, state management (Actix actors, WebSocket, HTTP).
- **Game Logic Layer (`src/game/`)**: Core game rules, entities, grid, systems, and utilities.

See [Architecture Overview](../docs/ARCHITECTURE.md) for a detailed description of the backend and its place in the global architecture.

---

## 📂 Directory Structure

```
src/
|-- config/           # Game and matchmaking constants
|-- server/           # Networking, matchmaking, game sessions (Actix actors)
|-- game/             # Game logic, entities, grid, systems
|-- utils/            # Utility functions
|-- main.rs           # Entry point
|-- Cargo.toml
```

---

## 🚀 Running the Backend

```bash
cargo run
```

The backend exposes WebSocket endpoints for matchmaking and game sessions.

---

## 🧩 Key Concepts

- **Actix actors** for concurrency and real-time networking.
- **WebSocket** for client-server communication.
- **Strong typing** (Rust enums, structs) for robust message passing.
- **Separation of concerns**: networking, game logic, and configuration are clearly separated.

---

## 📝 Code & Documentation Conventions

- All code and comments are in **English**.
- Use Rust doc comments (`///`) for all public items.
- Follow [Commenting Guidelines](../docs/COMMENTING_GUIDELINES.md) (Rust section).
- See [Contributing Guide](../CONTRIBUTING.md) for contribution rules.

---

## 🧪 Testing

Run all tests with:

```bash
cargo test
```

---

## 📚 Further Reading

- [Architecture](../docs/ARCHITECTURE.md)
- [API Reference](../docs/API.md)
- [Game Rules](../docs/GAME_RULES.md)
- [Commenting Guidelines](../docs/COMMENTING_GUIDELINES.md)

---

## 🤝 Contributing

See [Contributing Guide](../CONTRIBUTING.md).

---

## 📜 License

MIT

---

**Questions? Open an issue or discussion!**
