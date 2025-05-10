# Lava Grid 🔥

**ZK-powered strategy game built with Rust and Succinct.**  
This project explores trustless game logic using zero-knowledge proofs.

---

## ✨ What is Lava Grid?

Lava Grid is an on-chain strategy game experiment built in Rust.  
Using [Succinct](https://www.succinct.xyz/) and zero-knowledge proofs, we aim to create a game where moves can be **verified cryptographically** without revealing the entire game state.

The goal: **trustless, transparent, and provable gameplay**.

---

## 🛠️ Tech Stack

- 🦀 **Rust** — for performance and native zk integration
- 🧪 **Succinct** — zero-knowledge proof infra (SNARKs)
- 📦 `cargo`, `splup` — Rust + zk toolchain
- 💡 Future: frontend in Next.js + React (TBD)

---

## 🚀 Getting Started

### 1. Clone the repo

```bash
git clone https://github.com/your-username/lava-grid.git
cd lava-grid
```

### 2. Install Rust

If you don’t have Rust installed, you can install it by running:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the on-screen instructions to complete the installation.

### 3. Build & run

After cloning the repository, navigate to your project folder and run:

```bash
cargo run
```

### 4. Generate proof (example)

To generate a zero-knowledge proof, use the `splup` tool:

```bash
splup prove examples/fibonacci.spl
```

---

## 🧱 Project Structure

```
lava-grid/
├── program/          # Your core zk logic (as a Rust lib)
│   └── src/lib.rs
├── src/main.rs       # Entry point (calls program logic)
├── Cargo.toml
└── README.md
```

---

## 📜 License

MIT — feel free to use and build on it.

---

## 🤝 Contributing

Pull requests welcome! Let’s build the future of on-chain games together.

---

## 📬 Contact

DM me on X: [@mehdin_eth](https://x.com/mehdin_eth)  
Or open an issue / discussion!
