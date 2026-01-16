#!/usr/bin/env bash

set -e

# Cross-compilation script for flow_dashboard
# Builds the ruby FFI library for Linux and macOS targets

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_DIR="${PROJECT_ROOT}/target/cross-builds"

# Targets
LINUX_X86="x86_64-unknown-linux-gnu"
# LINUX_ARM="aarch64-unknown-linux-gnu"
# MACOS_X86="x86_64-apple-darwin"
MACOS_ARM="aarch64-apple-darwin"

mkdir -p "$OUTPUT_DIR"

echo "=== Cross-compilation build script ==="
echo "Project root: $PROJECT_ROOT"
echo "Output directory: $OUTPUT_DIR"
echo ""

# Check if cross is installed
if ! command -v cross &>/dev/null; then
  echo "Error: 'cross' is not installed."
  echo "Install it with: cargo install cross"
  exit 1
fi

build_with_cross() {
  local target="$1"
  echo "--- Building for $target with cross ---"
  (cd "$PROJECT_ROOT" && cross build --release --target="$target")

  # Copy artifacts to output directory
  local ext
  if [[ "$target" == *"linux"* ]]; then
    ext="so"
  else
    ext="dylib"
  fi

  local artifact="$PROJECT_ROOT/target/$target/release/saless_app.$ext"
  if [[ -f "$artifact" ]]; then
    cp "$artifact" "$OUTPUT_DIR/saless_app.$ext"
    echo "Copied: saless_app.$ext"
  else
    echo "Warning: artifact not found at $artifact"
  fi
}

build_with_cargo() {
  local target="$1"
  echo "--- Building for $target with cargo ---"
  (cd "$PROJECT_ROOT" && cargo build --release --target="$target")

  local ext
  if [[ "$target" == *"linux"* ]]; then
    ext="so"
  else
    ext="dylib"
  fi

  local artifact="$PROJECT_ROOT/target/$target/release/saless_app.$ext"
  if [[ -f "$artifact" ]]; then
    cp "$artifact" "$OUTPUT_DIR/saless_app.$ext"
    echo "Copied: saless_app.$ext"
  else
    echo "Warning: artifact not found at $artifact"
  fi
}

# Parse arguments
BUILD_ALL=false
BUILD_LINUX=false
BUILD_MACOS=false

if [[ $# -eq 0 ]]; then
  BUILD_ALL=true
else
  for arg in "$@"; do
    case "$arg" in
    --all)
      BUILD_ALL=true
      ;;
    --linux)
      BUILD_LINUX=true
      ;;
    --macos)
      BUILD_MACOS=true
      ;;
    --help | -h)
      echo "Usage: $0 [options]"
      echo ""
      echo "Options:"
      echo "  --all     Build for all targets (default if no options)"
      echo "  --linux   Build for Linux targets (x86_64, aarch64)"
      echo "  --macos   Build for macOS targets (x86_64, aarch64)"
      echo "  --help    Show this help message"
      echo ""
      echo "Note: macOS targets can only be built on macOS."
      echo "      Linux targets use 'cross' for cross-compilation."
      exit 0
      ;;
    *)
      echo "Unknown option: $arg"
      exit 1
      ;;
    esac
  done
fi

if "$BUILD_ALL"; then
  BUILD_LINUX=true
  BUILD_MACOS=true
fi

# Build Linux targets (using cross)
if "$BUILD_LINUX"; then
  echo ""
  echo "=== Building Linux targets ==="
  build_with_cross "$LINUX_X86"
  # build_with_cross "$LINUX_ARM"
fi

# Build macOS targets (native cargo, only works on macOS)
if "$BUILD_MACOS"; then
  echo ""
  echo "=== Building macOS targets ==="
  if [[ "$(uname)" != "Darwin" ]]; then
    echo "Warning: macOS targets can only be built on macOS. Skipping."
  else
    # Check if targets are installed
    # if ! rustup target list --installed | grep -q "$MACOS_X86"; then
    #   echo "Installing $MACOS_X86 target..."
    #   rustup target add "$MACOS_X86"
    # fi
    if ! rustup target list --installed | grep -q "$MACOS_ARM"; then
      echo "Installing $MACOS_ARM target..."
      rustup target add "$MACOS_ARM"
    fi

    # build_with_cargo "$MACOS_X86"
    build_with_cargo "$MACOS_ARM"
  fi
fi

echo ""
echo "=== Build complete ==="
echo "Artifacts are in: $OUTPUT_DIR"
ls -la "$OUTPUT_DIR"
