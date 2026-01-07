#!/bin/bash
# Install git hooks for the project

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOKS_SOURCE="$PROJECT_ROOT/.git-hooks"
HOOKS_TARGET="$PROJECT_ROOT/.git/hooks"

echo "Installing git hooks..."

# Check if .git directory exists
if [ ! -d "$PROJECT_ROOT/.git" ]; then
    echo "Error: .git directory not found. Are you in a git repository?"
    exit 1
fi

# Create hooks directory if it doesn't exist
mkdir -p "$HOOKS_TARGET"

# Install pre-commit hook
if [ -f "$HOOKS_SOURCE/pre-commit" ]; then
    cp "$HOOKS_SOURCE/pre-commit" "$HOOKS_TARGET/pre-commit"
    chmod +x "$HOOKS_TARGET/pre-commit"
    echo "✓ Installed pre-commit hook"
else
    echo "Error: pre-commit hook not found in $HOOKS_SOURCE"
    exit 1
fi

echo ""
echo "✅ Git hooks installed successfully!"
echo ""
echo "The following checks will run before each commit:"
echo "  - cargo fmt (code formatting)"
echo "  - cargo clippy (linting)"
echo ""
echo "To bypass these checks (not recommended), use: git commit --no-verify"

