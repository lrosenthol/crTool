#!/bin/bash
# Quick verification script for GUI setup

echo "Verifying crTool GUI Setup..."
echo ""

# Check if library source exists
if [ -f "src/lib.rs" ]; then
    echo "✓ Library API (src/lib.rs) exists"
else
    echo "✗ Missing: src/lib.rs"
    exit 1
fi

# Check if GUI directory exists
if [ -d "crtool-gui" ]; then
    echo "✓ GUI directory (crtool-gui/) exists"
else
    echo "✗ Missing: crtool-gui/"
    exit 1
fi

# Check if GUI source exists
if [ -f "crtool-gui/src/main.rs" ]; then
    echo "✓ GUI source (crtool-gui/src/main.rs) exists"
else
    echo "✗ Missing: crtool-gui/src/main.rs"
    exit 1
fi

# Check if workspace Cargo.toml exists
if [ -f "Cargo.toml" ]; then
    echo "✓ Workspace Cargo.toml exists"
else
    echo "✗ Missing: Cargo.toml"
    exit 1
fi

# Check if schema exists
if [ -f "INTERNAL/schemas/indicators-schema.json" ]; then
    echo "✓ Indicators schema exists"
else
    echo "✗ Missing: INTERNAL/schemas/indicators-schema.json"
    exit 1
fi

echo ""
echo "All files present!"
echo ""
echo "Next steps:"
echo "1. Build the GUI: cargo build --release -p crTool-gui"
echo "2. Run the GUI: cargo run --release -p crTool-gui"
echo "3. Test with files from testset/ directory"
