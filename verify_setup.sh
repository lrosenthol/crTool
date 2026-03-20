#!/bin/bash
# Quick verification script to ensure the repository is set up correctly

set -e

echo "🔍 Verifying Content Credential Tool repository setup..."
echo ""

# Check Rust is installed
echo "✓ Checking Rust installation..."
if ! command -v rustc &> /dev/null; then
    echo "❌ Rust is not installed. Please install from https://rustup.rs/"
    exit 1
fi
echo "  Rust version: $(rustc --version)"

# Check c2pa-rs dependency exists
echo ""
echo "✓ Checking c2pa-rs dependency..."
if [ ! -d "../c2pa-rs/sdk" ]; then
    echo "❌ c2pa-rs repository not found at ../c2pa-rs/"
    echo "   Please clone it:"
    echo "   cd .. && git clone https://github.com/contentauth/c2pa-rs.git"
    exit 1
fi
echo "  Found c2pa-rs SDK at ../c2pa-rs/sdk"

# Check test files exist
echo ""
echo "✓ Checking test files..."
if [ ! -f "tests/fixtures/assets/raw/Dog.jpg" ] || [ ! -f "tests/fixtures/assets/raw/Dog.png" ] || [ ! -f "tests/fixtures/assets/raw/Dog.webp" ]; then
    echo "❌ Test image files missing in tests/fixtures/assets/raw/"
    exit 1
fi
echo "  All test images present"

# Check test certificates exist
echo ""
echo "✓ Checking test certificates..."
if [ ! -f "tests/fixtures/certs/ed25519.pub" ] || [ ! -f "tests/fixtures/certs/ed25519.pem" ]; then
    echo "❌ Test certificates missing in tests/fixtures/certs/"
    exit 1
fi
echo "  Test certificates present"

# Check example manifests exist
echo ""
echo "✓ Checking example manifests..."
if [ ! -f "examples/simple_manifest.json" ] || [ ! -f "examples/full_manifest.json" ]; then
    echo "❌ Example manifests missing in examples/"
    exit 1
fi
echo "  Example manifests present"

# Try to build
echo ""
echo "✓ Building project..."
if ! cargo build --quiet 2>&1 | grep -q "Finished"; then
    cargo build
fi
echo "  Build successful"

# Run tests
echo ""
echo "✓ Running tests..."
if cargo test --quiet -- --nocapture 2>&1 | tail -1 | grep -q "ok"; then
    TEST_OUTPUT=$(cargo test --quiet 2>&1 | grep "test result")
    echo "  $TEST_OUTPUT"
else
    echo "❌ Tests failed. Run 'cargo test' for details."
    exit 1
fi

echo ""
echo "✅ All checks passed! Repository is properly set up."
echo ""
echo "Quick commands:"
echo "  cargo build --release      # Build release binary"
echo "  cargo test                 # Run all tests"
echo "  ./generate_test_certs.sh   # Generate test certificates"
echo ""
echo "Binary location: target/release/crTool"
