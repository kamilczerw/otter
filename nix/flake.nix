{
  description = "Development environment for Otter Budget Tracker";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      crane,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let

        pkgs = import nixpkgs {
          inherit system;
        };

        craneLib = crane.mkLib pkgs;

        # Filter source to only include relevant files for Rust builds
        src = craneLib.cleanCargoSource (craneLib.path ../backend);

        # Common arguments for crane
        commonArgs = {
          inherit src;
          strictDeps = true;

          pname = "squirrel-backend";
          version = "0.1.0";

          buildInputs = with pkgs; [
            openssl
          ];

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          # Use SQLx offline mode (requires sqlx-data.json)
          SQLX_OFFLINE = "true";
        };

        # Build dependencies separately for better caching
        cargoArtifacts = craneLib.buildDepsOnly (
          commonArgs
          // {
            pname = "squirrel-backend-deps";
          }
        );

        # Build the actual backend
        backend = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;

            # Build only the otter binary
            cargoExtraArgs = "--bin otter";
          }
        );

        # Run backend tests with cargo-nextest
        backend-tests = craneLib.cargoNextest (
          commonArgs
          // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          }
        );

        # Run backend tests with regular cargo test
        backend-tests-cargo = craneLib.cargoTest (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );

      in
      {
        packages = {
          inherit backend backend-tests backend-tests-cargo;
          default = backend;
        };

        checks = {
          inherit backend;
        };

        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            nodePackages.prettier
            nodejs_24
            just # Task runner (evolution of make)

            figlet # Print the welcome message as ascii art
            lolcat # Make the ascii art colorful

            cargo-watch # Automatically compile the project when a file changes, usage: `cargo watch -x run`
            cargo-nextest # Test runner
            cargo-chef # Build dependendies to speed up docker build
            cargo-deny
            cargo-nextest # Cargo test runner

            rust-analyzer
            rustup

            openssl
            pkg-config

          ];

          env = {
            OPENSSL_DIR = pkgs.openssl.dev;
            OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
            OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
          };

          shellHook = ''
            LOCAL_DIR="$(pwd)/.local"

            # Add the npm global bin directory to PATH
            export PATH="$NPM_CONFIG_PREFIX/bin:$PATH"

            export CARGO_HOME="$(pwd)/.local/cargo"
            export CARGO_TARGET_DIR="$(pwd)/.local/cargo"
            mkdir -p "$CARGO_HOME"

            export PATH="$CARGO_HOME/bin:$NPM_CONFIG_PREFIX/bin:$PATH"

            figlet -f slant "Otter!" -t | lolcat
          '';
        };
      }
    );
}
