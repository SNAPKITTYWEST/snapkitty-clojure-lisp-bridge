#!/bin/bash
# Build NASM ASM + Node.js C++ binding for SKC-LISP validators

set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_DIR="$DIR/build"

echo "[SKC-LISP] Building native validators..."

# Create build directory
mkdir -p "$BUILD_DIR/Release"

# Assemble NASM modules
echo "[SKC-LISP] Assembling NASM modules..."
nasm -f elf64 -o "$BUILD_DIR/Release/mutation-validator.o" "$DIR/mutation-validator.asm"
nasm -f elf64 -o "$BUILD_DIR/Release/digest-verifier.o" "$DIR/digest-verifier.asm"

# Link into shared library
echo "[SKC-LISP] Linking ASM library..."
ld -shared -o "$BUILD_DIR/Release/libskclisp_asm.so" \
  "$BUILD_DIR/Release/mutation-validator.o" \
  "$BUILD_DIR/Release/digest-verifier.o"

# Build Node.js addon with node-gyp
echo "[SKC-LISP] Building Node.js addon..."
cd "$DIR"
node-gyp configure
node-gyp build --debug
# or: node-gyp build (for release)

echo "[SKC-LISP] Build complete: $BUILD_DIR/Release/skclisp_native.node"
echo "[SKC-LISP] ASM library: $BUILD_DIR/Release/libskclisp_asm.so"
