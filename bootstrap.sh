#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_NAME="iggy"
CRATE_DIR="$SCRIPT_DIR/$CRATE_NAME"
GRAMMAR_FILE="$CRATE_DIR/iggy.iggy"

# Run the generator with output directly to the sub-crate
echo "Running parser generator..."
cargo run -p iguana -- generate --grammar "$GRAMMAR_FILE" --output "$CRATE_DIR"

echo "The parser for Iggy has been successfully generated."
