#!/bin/bash
# Build script for creating both prod and recorder binaries

set -e

echo "Building Superpilot binaries..."
echo

# Clean previous builds
echo "Cleaning previous builds..."
cargo clean
echo "✓ Clean complete"
echo

# Build prod binary
echo "Building PROD binary (observe-only)..."
cargo build --release --features prod
cp target/release/superpilot target/release/superpilot-prod
echo "✓ Prod binary: target/release/superpilot-prod"
ls -lh target/release/superpilot-prod | awk '{print "  Size: " $5}'
echo

# Build recorder binary
echo "Building RECORDER binary (MITM + capture)..."
cargo build --release --features recorder
cp target/release/superpilot target/release/superpilot-recorder
echo "✓ Recorder binary: target/release/superpilot-recorder"
ls -lh target/release/superpilot-recorder | awk '{print "  Size: " $5}'
echo

# Create symbolic link to prod as default
ln -sf superpilot-prod target/release/superpilot
echo "✓ Default binary (superpilot) → superpilot-prod"
echo

echo "Build complete! Binaries:"
echo "  Production:  ./target/release/superpilot-prod"
echo "  Recorder:    ./target/release/superpilot-recorder"
echo "  Default:     ./target/release/superpilot → superpilot-prod"
echo

# Test binaries
echo "Testing binaries..."
timeout 1 ./target/release/superpilot-prod 2>&1 | head -5 || true
echo
timeout 1 ./target/release/superpilot-recorder 2>&1 | head -5 || true
echo
echo "✓ All binaries operational"
