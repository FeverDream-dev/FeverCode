# FeverCode Installation Guide

Purpose
- Detailed, honest steps to install FeverCode from source, with prerequisites, build, and PATH setup.

Current status: Working. Cargo fmt/clippy/test pass, and the build artifacts are produced as expected. Binary naming is project-configurable; see Cargo.toml for exact binary name.

Prerequisites
- Rust 1.70+ toolchain (rustc --version)
- Git (git --version)
- A Unix-like shell (Linux/macOS) or Windows Subsystem for Linux (recommended for local builds)

Clone the repository
- git clone https://github.com/FeverDream-dev/FeverCode fevercode_starter
- cd fevercode_starter
- Note: The repo layout exposes a top-level package FeverCode in this directory. Run commands from this root.

Build
- cargo fmt --all
- cargo build --release
- cargo test --tests

Binary names and PATH
- The build outputs a binary under target/release. The exact binary name is defined in Cargo.toml; typically it is fevercode or fevercode-bin.
- Add the release binary directory to your PATH, for example:
  - export PATH="$PATH:$(pwd)/target/release"
- Verify by running the binary name (e.g., fevercode --version) if supported by the project.

Post-install verification
- cargo fmt --all --check
- cargo clippy --all-targets --all-features -- -D warnings
- cargo test --tests

Troubleshooting
- If the build fails, make sure you have a recent Rust toolchain and a clean workspace (cargo clean) before retrying.
- Ensure dependencies are up to date: cargo update
- If the binary name differs, search the Cargo.toml for the [[bin]] section to confirm the executable name.
