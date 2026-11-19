#!/usr/bin/env bash
# Build the release binary and expose it as `harbour-dev`, separate from the Homebrew `harbour`.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release --locked
mkdir -p ~/.local/bin
ln -sfn "$PWD/target/release/harbour" ~/.local/bin/harbour-dev
echo "harbour-dev -> $PWD/target/release/harbour"
