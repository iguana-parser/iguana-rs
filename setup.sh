#!/bin/bash
set -e

# Toolchain
command -v cargo >/dev/null || { echo "Install rust: https://rustup.rs"; exit 1; }

# Dev tools
cargo install cargo-nextest --locked

# System deps
command -v dot >/dev/null || echo "Note: graphviz (dot) not installed; visualization tasks will fail. Install with: brew install graphviz"

# Git hooks
git config core.hooksPath .githooks
echo "Hooks: $(git config --get core.hooksPath)"

echo "Setup complete."
