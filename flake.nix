{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            pkg-config
            openssl
          ];
          shellHook = ''
            export PATH="$HOME/.risc0/bin:$PATH"
          '';
        };

        # SP1 needs rustup so cargo-prove can register the succinct toolchain.
        # Keep this separate from `default` to avoid clashing with rust-overlay.
        devShells.sp1 = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustup
            pkg-config
            openssl
            git
            curl
            gcc
            protobuf
            # SP1 from-source builds (cargo-prove etc.) pull in bindgen for the
            # gnark Go FFI, which needs libclang.
            llvmPackages.libclang
            go
          ];
          shellHook = ''
            export PATH="$HOME/.sp1/bin:$PATH"
            export RUSTUP_HOME="$HOME/.rustup"
            export CARGO_HOME="$HOME/.cargo"
            export PATH="$CARGO_HOME/bin:$PATH"
            # nix-ld lets glibc-linked binaries (sp1up's cargo-prove) run.
            export NIX_LD="''${NIX_LD:-/run/current-system/sw/share/nix-ld/lib/ld.so}"
            export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"
          '';
        };
      }
    );
}
