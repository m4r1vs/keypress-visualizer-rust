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

The application uses a TOML configuration file to map raw key names (e.g., `LEFTMETA`) to display strings (e.g., `LCMD`).

On NixOS, the preferred way to configure the application is through the NixOS module's `settings` option (see the **NixOS Module** section below), which automatically generates `/etc/keypress-visualizer-rust/default_config.toml`.

For other systems, it looks for `default_config.toml` in the current working directory or via the `KEYPRESS_VISUALIZER_CONFIG` environment variable.

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

## NixOS Module

This flake provides a NixOS module that simplifies installation and handles permissions automatically.

To use it, add the flake to your inputs and enable the module:

```nix
{
  programs.keypress-visualizer.enable = true;
  # Optional: custom settings
  programs.keypress-visualizer.settings = {
    appearance.font_size = 30;
  };
}
```

The module automatically sets up a security wrapper with `cap_dac_override` and `cap_sys_ptrace` so the program can read `/dev/input` without root or special group memberships.

## Desktop Integration

A `.desktop` file is included and installed to `share/applications`. On NixOS, enabling the module will add "Keypress Visualizer" to your application menu.

## Development Conventions

- **Input Discovery:** The application currently looks for keyboards matching `/dev/input/by-id/usb-*event-kbd`.
- **Styling:** CSS styling is embedded in `src/main.rs`.
- **Lifetimes:** Keypress labels are automatically removed from the UI after 2 seconds.
- **Error Handling:** Errors in the input loop are printed to `stderr`.

## Troubleshooting

- **No Keyboard Found:** Ensure your keyboard is identified correctly under `/dev/input/by-id/`. You might need to adjust the pattern in `src/input.rs`.
- **Permission Denied:** The application needs to read `/dev/input/event*`. Running with the `dev` script in `nix develop` handles this via capabilities, or you can run with `sudo`.
