# 📁 TUI File Explorer

A simple terminal-based file manager written in Rust using the [Ratatui](https://github.com/ratatui-org/ratatui)  library.

## 🧰 Description
This is a console application that lets you browse your file system, navigate through directories, and select a path. Once exited (by pressing **Enter**), the selected path is printed to the terminal.

## 🖥️ Features
- View files and directories
- Navigation:
  - **→ (Right)** — open directory
  - **← (Left)** — go back
  - **↑ / ↓** — navigate items
  - **Enter** — select current item and exit
  - **q** — quit without selection
- File type-dependent icons
- Responsive layout based on terminal size
