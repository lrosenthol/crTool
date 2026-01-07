#!/bin/bash
# Format all Rust code in the project

set -e

echo "Formatting Rust code..."
cargo fmt --all

echo ""
echo "✅ Code formatted successfully!"
echo ""
echo "Run 'git diff' to see the changes."

