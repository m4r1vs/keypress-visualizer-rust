{
  description = "Linux keypress overlay dev flake";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-25.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    ...
  }: let
    supportedSystems = [
      "x86_64-linux"
      "aarch64-linux"
    ];
    forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
  in {
    nixosModules.default = {
      config,
      lib,
      pkgs,
      ...
    }: let
      cfg = config.programs.keypress-visualizer;
    in {
      options.programs.keypress-visualizer = {
        enable = lib.mkEnableOption "Keypress Visualizer";
        package = lib.mkOption {
          type = lib.types.package;
          default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
          description = "The keypress-visualizer package to use.";
        };
        useWrapper = lib.mkOption {
          type = lib.types.bool;
          default = true;
          description = ''
            Whether to use a security wrapper with necessary capabilities.
            This allows the program to read input devices without being root or in the input group.
          '';
        };
        settings = lib.mkOption {
          type = lib.types.submodule {
            freeformType = (pkgs.formats.toml {}).type;
          };
          default = {};
          description = "Configuration for keypress-visualizer-rust.";
        };
      };

      config = lib.mkIf cfg.enable {
        environment.systemPackages = [cfg.package];

        environment.etc."keypress-visualizer-rust/default_config.toml" = lib.mkIf (cfg.settings != {}) {
          source = (pkgs.formats.toml {}).generate "keypress-visualizer-config.toml" cfg.settings;
        };

        security.wrappers.keypress-visualizer-rust = lib.mkIf cfg.useWrapper {
          owner = "root";
          group = "root";
          capabilities = "cap_dac_override,cap_sys_ptrace+ep";
          source = "${cfg.package}/bin/keypress-visualizer-rust";
        };
      };
    };

    packages = forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };
      rustPlatform = pkgs.makeRustPlatform {
        cargo = pkgs.rust-bin.stable.latest.default;
        rustc = pkgs.rust-bin.stable.latest.default;
      };
      runtimeDeps = with pkgs; [
        gtk4
        gtk4-layer-shell
        glib
        adwaita-icon-theme
      ];
    in {
      default = rustPlatform.buildRustPackage {
        pname = "keypress-visualizer-rust";
        version = "0.1.0";
        src = ./.;

        cargoLock = {
          lockFile = ./Cargo.lock;
        };

        nativeBuildInputs = with pkgs; [
          pkg-config
          makeWrapper
        ];

        buildInputs = runtimeDeps;

        postInstall = ''
          mkdir -p $out/share/keypress-visualizer-rust
          cp default_config.toml default_style.css $out/share/keypress-visualizer-rust/

          mkdir -p $out/share/applications
          cp keypress-visualizer.desktop $out/share/applications/

          wrapProgram $out/bin/keypress-visualizer-rust \
            --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath runtimeDeps}" \
            --prefix XDG_DATA_DIRS : "$GSETTINGS_SCHEMAS_PATH" \
            --set KEYPRESS_VISUALIZER_CONFIG "$out/share/keypress-visualizer-rust/default_config.toml"
        '';

        meta = with pkgs.lib; {
          description = "A simple program to show keypresses";
          homepage = "https://github.com/m4r1vs/keypress-visualiser-rust";
          license = licenses.mit;
          platforms = platforms.linux;
          mainProgram = "keypress-visualizer-rust";
        };
      };
    });

    devShells = forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };
      runtimeDeps = with pkgs; [
        gtk4
        gtk4-layer-shell
        glib
      ];
      runScript = pkgs.writeShellScriptBin "dev" ''
        cargo build
        sudo setcap 'cap_dac_override,cap_sys_ptrace+ep' ./target/debug/keypress-visualizer-rust
        ./target/debug/keypress-visualizer-rust
      '';
    in {
      default = pkgs.mkShell {
        buildInputs = with pkgs;
          [
            (rust-bin.stable.latest.default.override {
              extensions = ["rust-src"];
            })
            pkg-config
            runScript
          ]
          ++ runtimeDeps;

        shellHook = ''
          export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath runtimeDeps}:$LD_LIBRARY_PATH"
        '';
      };
    });
  };
}
