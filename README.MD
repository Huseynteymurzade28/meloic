# Meloic 🎵

Meloic is a modern Terminal User Interface (TUI) music player built with Rust. Manage your music collection through the terminal with a lightweight, fast, and user-friendly interface.

## ✨ Features

- 🎵 **Intuitive Music Selection**: Easily browse through your music collection
- ⏸️ **P Key Pause**: Instantly pause/resume music playback
- ⏯️ **Enter to Play**: Start playing the selected track
- 🔄 **Navigate Between Tracks**: Move between songs using arrow keys
- 🚀 **Fast and Lightweight**: Leverages Rust's performance for smooth experience
- 💻 **Terminal-Based**: Works without the need for GUI

## 📦 Installation

### Install via Cargo (Recommended)

```bash
cargo install meloic
```

### Build from Source

```bash
git clone https://github.com/Huseynteymurzade28/meloic.git
cd meloic
cargo build --release
```

You can find the compiled binary at `target/release/meloic`.

## 🚀 Usage

To run Meloic, open your terminal and type:

```bash
meloic
```

### Controls

| Key     | Description             |
| ------- | ----------------------- |
| `↑/↓`   | Navigate between tracks |
| `Enter` | Play selected track     |
| `p`     | Pause/resume playback   |
| `q`     | Quit application        |

## 🏗️ Project Structure

```
src/
├── ui/
│   ├── mod.rs      # UI module definitions
│   ├── events.rs   # Event handling (key presses, etc.)
│   ├── app.rs      # Main application logic
│   └── widgets.rs  # UI components
├── library.rs      # Music library management
└── main.rs         # Application entry point
Cargo.toml         # Project configuration and dependencies
Cargo.lock        # Locked dependencies
README.MD          # Project documentation
.gitignore        # Ignored files for git
```

## 🛠️ Development

To develop the project:

```bash
git clone https://github.com/Huseynteymurzade28/meloic.git
cd meloic
cargo run
```

## 📋 Requirements

- Rust 1.70.0 or higher
- Cargo package manager

## 🤝 Contributing

We welcome contributions! Please:

1. Fork the project
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the [MIT License](LICENSE).

## 📞 Contact

For questions or suggestions:

- GitHub Issues: [Issues page](https://github.com/yourusername/meloic/issues)

---

_Music in its purest form. In the terminal, simple and powerful._ 🎵
