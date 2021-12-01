# Rust-Nix Template

Repository template for a Rust project that pulls in the rust compiler (and
cargo etc.) via nix. The `shell.nix` file provides an environment containing
`rustc` and `cargo`, as well as the libstd source code for facilitating the use
of [rust-analyzer](https://github.com/rust-analyzer/rust-analyzer).

Rust-Analyzer itself is *not* provided via the nix environment, as the official
Rust-Analyzer plugin for VS Code already takes care of downloading the latest
version (and even patches the binary for NixOS).

## Getting Started

Simply spawn a nix shell by running `nix-shell` in this directory. Inside that
shell, `rustc` and some convenience tools are available. Instead of running
`nix-shell` manually, [direnv](https://direnv.net/) can improve usability. A
suitable `.envrc` is included.

For use with **VS Code** it is important to start VS Code from within the
`nix-shell` session by running `code .`. That way, it will capture the
environment of the nix shell. Otherwise, it won't be able to find rustc and the
libstd sources.

## Updating Upstream

The nix dependencies are managed using [niv](https://github.com/nmattia/niv).
Niv is provided via the `shell.nix` environment. In order to get the latest
snapshot, just run `niv update` inside the nix shell.
