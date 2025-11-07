#!/bin/bash
# Build script for Linux/macOS - Heap Allocator Tests
#
# Usage:
#   ./build_allocator_tests.sh         Build test executable
#   ./build_allocator_tests.sh test    Build and run tests
#   ./build_allocator_tests.sh clean   Clean build artifacts

set -e

echo "╔════════════════════════════════════════════════════════════╗"
echo "║  Glimmer-Weave Allocator Test Builder (Unix)              ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Check for GCC
if ! command -v gcc &> /dev/null; then
    echo "❌ ERROR: GCC not found in PATH"
    echo ""
    echo "Please install GCC:"
    echo "  - Ubuntu/Debian: sudo apt install gcc"
    echo "  - Fedora/RHEL:   sudo dnf install gcc"
    echo "  - macOS:         xcode-select --install"
    echo ""
    exit 1
fi

case "${1:-build}" in
    clean)
        echo "Cleaning build artifacts..."
        rm -f test_allocator
        echo "✅ Clean complete"
        ;;

    test)
        echo "Building allocator tests..."
        echo ""
        gcc -Wall -Wextra -g -O2 -o test_allocator test_allocator.c ../src/native_allocator.S
        echo ""
        echo "✅ Build complete: test_allocator"
        echo ""
        echo "Running tests..."
        echo ""
        ./test_allocator
        TEST_RESULT=$?
        echo ""
        if [ $TEST_RESULT -eq 0 ]; then
            echo "✅ All tests passed!"
        else
            echo "❌ Tests failed with exit code $TEST_RESULT"
        fi
        exit $TEST_RESULT
        ;;

    build|*)
        echo "Building allocator tests..."
        echo ""
        gcc -Wall -Wextra -g -O2 -o test_allocator test_allocator.c ../src/native_allocator.S
        echo ""
        echo "✅ Build complete: test_allocator"
        echo ""
        echo "Run with: ./test_allocator"
        ;;
esac
