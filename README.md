## Configuration

We provide a `default_config.toml` as reference:

```toml
[mappings]
LEFTMETA = "⌘"
RIGHTMETA = "⌘"
LEFTSHIFT = "⇧"
RIGHTSHIFT = "⇧"
LEFTCTRL = "^"
RIGHTCTRL = "^"
LEFTALT = "⎇"
RIGHTALT = "⎇"
ENTER = "↵"
BACKSPACE = "⌫"
SPACE = "␣"
TAB = "↹"
ESC = "ESC"
CAPSLOCK = "CAPS"

[appearance]
font_size = 52
anchor = "top"
margin_x = 0
margin_y = 50
pos_x_pct = 0.0
pos_y_pct = 0.0
max_keys = 2
custom_css = ""
```

## Running the Visualizer

A development shell is defined in `flake.nix`, you can enter it using direnv or by running `nix develop`.

```bash
nix develop
dev
```

On other distros, install the rust toolchain from your package manager and then build and run manually:

```bash
cargo build
sudo setcap \
	'cap_dac_override,cap_sys_ptrace+ep' \
	./target/debug/keypress-visualizer-rust
```
