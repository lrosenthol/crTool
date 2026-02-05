#!/bin/bash
# Build script for crTool workspace

set -e  # Exit on error

echo "================================"
echo "Building crTool Workspace"
echo "================================"
echo ""

# Parse command line arguments
BUILD_CLI=true
BUILD_GUI=false
RELEASE_MODE=""

for arg in "$@"; do
    case $arg in
        --gui-only)
            BUILD_CLI=false
            BUILD_GUI=true
            shift
            ;;
        --cli-only)
            BUILD_CLI=true
            BUILD_GUI=false
            shift
            ;;
        --all)
            BUILD_CLI=true
            BUILD_GUI=true
            shift
            ;;
        --release)
            RELEASE_MODE="--release"
            shift
            ;;
        --help)
            echo "Usage: ./build.sh [options]"
            echo ""
            echo "Options:"
            echo "  --cli-only    Build only the CLI tool (default)"
            echo "  --gui-only    Build only the GUI tool"
            echo "  --all         Build both CLI and GUI"
            echo "  --release     Build in release mode"
            echo "  --help        Show this help message"
            echo ""
            exit 0
            ;;
        *)
            echo "Unknown option: $arg"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Show build configuration
if [ "$BUILD_CLI" = true ] && [ "$BUILD_GUI" = true ]; then
    echo "Building: CLI + GUI"
elif [ "$BUILD_CLI" = true ]; then
    echo "Building: CLI only"
else
    echo "Building: GUI only"
fi

if [ -n "$RELEASE_MODE" ]; then
    echo "Mode: Release"
else
    echo "Mode: Debug"
fi
echo ""

# Build CLI
if [ "$BUILD_CLI" = true ]; then
    echo "→ Building CLI tool..."
    cargo build $RELEASE_MODE
    echo "✓ CLI build complete"
    echo ""
fi

# Build GUI
if [ "$BUILD_GUI" = true ]; then
    echo "→ Building GUI tool..."
    cargo build $RELEASE_MODE -p crtool-gui
    echo "✓ GUI build complete"
    echo ""
fi

# Show output locations
echo "================================"
echo "Build Complete!"
echo "================================"
echo ""

if [ -n "$RELEASE_MODE" ]; then
    BUILD_DIR="target/release"
else
    BUILD_DIR="target/debug"
fi

if [ "$BUILD_CLI" = true ]; then
    echo "CLI binary: $BUILD_DIR/crtool"
fi

if [ "$BUILD_GUI" = true ]; then
    echo "GUI binary: $BUILD_DIR/crtool-gui"
fi

echo ""
echo "To run the CLI: ./$BUILD_DIR/crtool --help"
if [ "$BUILD_GUI" = true ]; then
    echo "To run the GUI: ./$BUILD_DIR/crtool-gui"
fi
