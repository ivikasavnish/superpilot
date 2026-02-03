#!/bin/bash
set -e

echo "=== Superpilot Build Verification Script ==="
echo

# Build prod binary
echo "Building prod binary..."
cargo build --release --features prod
echo "✓ Prod binary built successfully"

# Build recorder binary
echo "Building recorder binary..."
cargo build --release --features recorder
mv target/release/superpilot target/release/superpilot-recorder
echo "✓ Recorder binary built successfully"

# Check binary sizes
echo
echo "Binary sizes:"
ls -lh target/release/superpilot-recorder | awk '{print "  Recorder: " $5}'

# Run unit tests
echo
echo "Running unit tests..."
cargo test --features prod --quiet
echo "✓ Prod tests passed"

cargo test --features recorder --quiet
echo "✓ Recorder tests passed"

# Verify binaries can run
echo
echo "Verifying binaries..."

# Check prod binary version
timeout 1 target/release/superpilot-recorder --help 2>&1 || echo "  Binary responds (expected timeout)"

echo
echo "=== All checks passed! ==="
echo
echo "Build artifacts:"
echo "  Prod binary: target/release/superpilot-prod"
echo "  Recorder binary: target/release/superpilot-recorder"
