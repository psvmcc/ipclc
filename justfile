set shell := ["zsh", "-cu"]

# Show available commands.
default:
    @just --list

# Check formatting without changing files.
fmt-check:
    cargo fmt --all -- --check

# Format the Rust source code.
fmt:
    cargo fmt --all

# Run Clippy with warnings treated as errors.
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run the complete test suite.
test:
    cargo test --all-targets --all-features

# Run formatting, linting, and tests.
check: fmt-check clippy test

# Build a release binary.
build:
    cargo build --release

# Run ipclc. Example: just run 192.168.1.1/24
run *args:
    cargo run -- {{args}}
