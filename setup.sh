#!/bin/bash
set -e

# Toolchain
command -v cargo >/dev/null || { echo "Install rust: https://rustup.rs"; exit 1; }

# Dev tools
cargo install cargo-nextest --locked

# Node deps (web viewer; also used by terrarium and the VS Code extension)
if command -v npm >/dev/null; then
  npm install
else
  echo "Note: npm not installed; web-viewer build (cargo xtask install) and terrarium will fail. Install with: brew install node"
fi

# System deps
command -v dot >/dev/null || echo "Note: graphviz (dot) not installed; visualization tasks will fail. Install with: brew install graphviz"

# Git hooks
git config core.hooksPath .githooks
echo "Hooks: $(git config --get core.hooksPath)"

echo "Setup complete."
