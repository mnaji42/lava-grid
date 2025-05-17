# Lava Grid 🔥

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/your-username/lava-grid/actions)
[![Coverage Status](https://img.shields.io/badge/coverage-100%25-brightgreen)](https://github.com/your-username/lava-grid/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**ZK-powered strategy game built with Rust and Succinct.**  
A trustless, on-chain game experiment leveraging zero-knowledge proofs.

---

## ✨ What is Lava Grid?

Lava Grid is an on-chain strategy game where every move is **cryptographically verified** using zero-knowledge proofs (ZK).  
The goal: **trustless, transparent, and provable gameplay** — no need to trust the server, all logic can be verified!

---

## 🛠️ Tech Stack

- 🦀 **Rust** — backend, game logic, and ZK integration
- 🧪 **Succinct** — zero-knowledge proof infra (SNARKs)
- ⚡ **Actix** — async actor framework for networking
- 💻 **Next.js + React** — frontend web client
- 📦 `cargo`, `splup` — Rust + zk toolchain

---

## ���� Monorepo Structure

```
lava-grid/
├── backend/           # Rust backend (game logic, networking, ZK)
│   ├── src/
│   ├── Cargo.toml
│   └── README.md
├── frontend/          # Web client (Next.js + React)
│   ├── src/
│   └── README.md
├── docs/              # Documentation (architecture, API, conventions, etc.)
│   ├── ARCHITECTURE.md
│   ├── API.md
│   ├── GAME_RULES.md
│   ├── COMMENTING_GUIDELINES.md
│   └── README.md
├── CONTRIBUTING.md
├── README.md          # (this file)
└── ...
```

---

## 🚀 Getting Started

### 1. Clone the repo

```bash
git clone https://github.com/your-username/lava-grid.git
cd lava-grid
```

### 2. Backend (Rust)

```bash
cd backend
cargo run
```

### 3. Frontend (Next.js)

```bash
cd frontend
npm install
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) in your browser.

---

## 📚 Documentation

- [Documentation Overview](docs/README.md)
- [Architecture](docs/ARCHITECTURE.md)
- [API Reference](docs/API.md)
- [Game Rules](docs/GAME_RULES.md)
- [Commenting Guidelines](docs/COMMENTING_GUIDELINES.md)
- [Contributing Guide](CONTRIBUTING.md)

---

## 🗂️ Subproject READMEs

- [Backend README](backend/README.md)
- [Frontend README](frontend/README.md)

---

## 🤝 Contributing

Pull requests are welcome!  
Please read [CONTRIBUTING.md](CONTRIBUTING.md) and [docs/COMMENTING_GUIDELINES.md](docs/COMMENTING_GUIDELINES.md) before submitting code.

---

## 📜 License

MIT — feel free to use and build on it.

---

## 📬 Contact

DM [@mehdin_eth](https://x.com/mehdin_eth) on X  
Or open an issue/discussion on GitHub.

---

**Let’s build the future of on-chain games together!**
