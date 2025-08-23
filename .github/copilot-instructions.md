# Copilot Instructions for litra-rs

This document provides instructions for AI agents to set up the development environment and run build processes for the litra-rs project.

## Project Overview

- **Language**: Rust
- **Purpose**: Control Logitech Litra lights from command line, MCP clients, and Rust applications
- **Key Dependencies**: hidapi (requires libudev-dev on Linux)

## Development Environment Setup

### System Dependencies

On Linux, install the required system library:
```bash
sudo apt-get update && sudo apt-get install -y libudev-dev
```

### Rust Toolchain

The project uses a specific Rust version with required components:
```bash
rustup override set 1.85.1
rustup component add clippy rustfmt
```

## Development Commands

### Build

Check the project builds correctly:
```bash
cargo check --locked --workspace --all-features --all-targets
```

Build release binary:
```bash
cargo build --release
```

### Testing

Run all tests:
```bash
cargo test
```

### Code Quality

Format code and check formatting:
```bash
cargo fmt --all
cargo fmt --all -- --check
```

Run linter with warnings treated as errors:
```bash
cargo clippy --locked --workspace --all-features --all-targets -- -D warnings
```

### Running the Application

After building, test the binary works:
```bash
./target/release/litra --help
```

## Notes for AI Agents

- The project requires `libudev-dev` on Linux systems before building
- Rust toolchain is pinned to version 1.85.1 (see `rust-toolchain.toml`)
- All clippy warnings must be resolved (treated as errors)
- The project has minimal unit tests but includes doc tests
- Pre-commit hooks are configured but not required for basic development
