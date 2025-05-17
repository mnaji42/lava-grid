# Lava Grid — Frontend

**Next.js + React client for Lava Grid: real-time, trustless, ZK-powered strategy game.**

---

## 🖥️ Overview

This is the web client for Lava Grid.  
It connects to the Rust backend via WebSocket for matchmaking and gameplay, and will eventually verify zero-knowledge proofs for trustless game logic.

---

## 🚀 Getting Started

### 1. Install dependencies

```bash
cd frontend
npm install
```

### 2. Run the development server

```bash
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) in your browser.

---

## 🗂️ Project Structure

```
frontend/
├── src/            # Application source code
│   └── app/        # Next.js app directory
├── public/         # Static assets
├── package.json
├── next.config.ts
└── README.md
```

---

## ⚙️ Environment Variables

Copy `.env.example` to `.env.local` and configure as needed.

---

## 📝 Code & Documentation Conventions

- All code and comments should be in **English**.
- See [Commenting Guidelines](../docs/COMMENTING_GUIDELINES.md) (JS/TS section coming soon).
- See [Contributing Guide](../CONTRIBUTING.md) for contribution rules.

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
