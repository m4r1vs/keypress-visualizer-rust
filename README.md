# Keypress Visualizer (Rust)

A simple Linux utility that displays keypresses in a floating overlay window. Built with Rust, GTK4, and `gtk4-layer-shell`.

## NixOS Installation & Configuration

If you are using NixOS, you can use the provided module by adding this flake to your configuration:

```nix
{
  inputs.keypress-visualizer.url = "github:m4r1vs/keypress-visualiser-rust";

  outputs = { self, nixpkgs, keypress-visualizer, ... }: {
    nixosConfigurations.my-machine = nixpkgs.lib.nixosSystem {
      modules = [
        keypress-visualizer.nixosModules.default
        {
          programs.keypress-visualizer = {
            enable = true;
            settings = {
              mappings = {
                LEFTMETA = "⌘";
                SPACE = "␣";
              };
              appearance = {
                font_size = 52;
                anchor = "top";
                max_keys = 2;
              };
            };
          };
        }
      ];
    };
  };
}
```

### Benefits of the NixOS Module
- **Automatic Permissions**: Sets up a security wrapper with `cap_dac_override` and `cap_sys_ptrace` so you don't need to run as root or manage the `input` group.
- **Desktop Entry**: Adds "Keypress Visualizer" to your application launcher automatically.
- **Declarative Config**: Configure all mappings and appearance settings directly in your Nix config.

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
