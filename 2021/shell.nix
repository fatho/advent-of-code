let
  sources = import ./nix/sources.nix;
  mozilla = import (sources.nixpkgs-mozilla + "/rust-overlay.nix");
  nixpkgs = import sources.nixpkgs {
    overlays = [mozilla];
  };
  channel = nixpkgs.rustChannelOf { rustToolchain = ./rust-toolchain; };
in
  nixpkgs.mkShell {
    name = "auto-dev";
    nativeBuildInputs = with nixpkgs; [
      # Rust core
      channel.rust
      # Neat helper tools
      cargo-asm
      cargo-audit
      cargo-edit
      cargo-flamegraph

      # Nix tools
      niv
    ];
    
    # Always enable rust backtraces in development shell
    RUST_BACKTRACE = "1";

    # Provide sources for rust-analyzer, because nixpkgs rustc doesn't include them in the sysroot
    RUST_SRC_PATH = "${channel.rust-src}/lib/rustlib/src/rust/library";
  }
