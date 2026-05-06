# Keypress Visualizer (Rust)

A simple Linux utility that displays keypresses in a floating overlay window. Built with Rust, GTK4, and `gtk4-layer-shell`.

## Project Overview

- **Core Technology:** Rust (2024 edition)
- **UI Toolkit:** [GTK4](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/)
- **Overlay Support:** [gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell) for Wayland overlay windows.
- **Input Capture:** [evdev](https://docs.rs/evdev/latest/evdev/) for reading raw input events from `/dev/input/`.
- **Environment Management:** [Nix Flakes](https://nixos.wiki/wiki/Flakes) and `direnv` for a reproducible development environment.

## Architecture

- **UI Layer (`src/main.rs`):** Sets up the GTK4 application, configures the layer shell (bottom-center anchor, overlay layer), and manages the display of keypress labels. It loads mappings from `default_config.toml`.
- **Input Layer (`src/input.rs`):** Scans `/dev/input/by-id/` for keyboard devices and spawns a dedicated thread to poll for key events.
- **Communication:** Uses `std::sync::mpsc` channels to pass key names from the input thread to the main GTK loop.

## Configuration

The application uses a `default_config.toml` file to map raw key names (e.g., `LEFTMETA`) to display strings (e.g., `LCMD`).

Example `default_config.toml`:
```toml
[mappings]
LEFTMETA = "LCMD"
SPACE = "SPC"
```

## Getting Started

### Prerequisites

- **Linux** (Wayland recommended for layer-shell support).
- **Nix** with Flake support (optional but recommended).
- **Permissions:** The application requires read access to `/dev/input/` devices.

### Development Environment

If you have Nix and `direnv` installed, the environment will be set up automatically. Otherwise, run:

```bash
nix develop
```

### Building and Running

The `flake.nix` provides a convenience script `dev` inside the development shell:

```bash
# Inside nix develop
dev
```

This script:
1. Performs `cargo build`.
2. Sets necessary capabilities (`cap_dac_override`, `cap_sys_ptrace`) on the binary to allow it to read input devices without full `sudo`.
3. Executes the application.

Manual build:
```bash
cargo build
```

## Development Conventions

- **Input Discovery:** The application currently looks for keyboards matching `/dev/input/by-id/usb-*event-kbd`.
- **Styling:** CSS styling is embedded in `src/main.rs`.
- **Lifetimes:** Keypress labels are automatically removed from the UI after 2 seconds.
- **Error Handling:** Errors in the input loop are printed to `stderr`.

## Troubleshooting

- **No Keyboard Found:** Ensure your keyboard is identified correctly under `/dev/input/by-id/`. You might need to adjust the pattern in `src/input.rs`.
- **Permission Denied:** The application needs to read `/dev/input/event*`. Running with the `dev` script in `nix develop` handles this via capabilities, or you can run with `sudo`.
