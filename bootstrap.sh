#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_NAME="iggy"
CRATE_DIR="$SCRIPT_DIR/$CRATE_NAME"

# Run the generator with output directly to the sub-crate
echo "Running parser generator..."
cargo run --manifest-path "$SCRIPT_DIR/Cargo.toml" -- generate --output "$CRATE_DIR"

echo "The parser for Iggy has been successfully generated."
