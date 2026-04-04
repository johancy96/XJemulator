#!/bin/bash
set -e

# Enter project root
cd "$(dirname "$0")/.."

echo "==> Compiling Release build..."
# Ensure CARGO_HOME is set if requested
cargo build --release

echo "==> Checking cargo-deb..."
if ! command -v cargo-deb &> /dev/null; then
    echo "Installing cargo-deb..."
    cargo install cargo-deb
fi

echo "==> Checking cargo-generate-rpm..."
if ! cargo --list | grep -q "generate-rpm"; then
    echo "Installing cargo-generate-rpm..."
    cargo install cargo-generate-rpm
fi

echo "==> Building .deb package..."
cargo deb

echo "==> Building .rpm package..."
cargo generate-rpm

echo
echo "==> Done! Packages can be found in:"
echo "    - target/debian/"
echo "    - target/generate-rpm/"
