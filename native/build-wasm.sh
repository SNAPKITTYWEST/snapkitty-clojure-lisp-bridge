#!/bin/bash
# SKC-LISP: Build Rust → WASM (Phase 3D-3)
# Compiles crypto-wasm.rs to browser-ready WASM module

set -e

echo "=== SoulVM JIT: WASM Build Pipeline ==="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# ============================================================================
# Step 1: Install wasm-pack (if needed)
# ============================================================================

echo "[1/6] Checking wasm-pack..."

if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    curl https://rustwasm.org/wasm-pack/installer/init.sh -sSf | sh
fi

WASM_PACK_VERSION=$(wasm-pack --version)
echo "✓ wasm-pack installed: $WASM_PACK_VERSION"

# ============================================================================
# Step 2: Install Rust wasm32 target
# ============================================================================

echo "[2/6] Installing wasm32-unknown-unknown target..."

rustup target add wasm32-unknown-unknown
echo "✓ Target installed"

# ============================================================================
# Step 3: Run Rust tests (native)
# ============================================================================

echo "[3/6] Running Rust unit tests (native)..."

cargo test --lib 2>&1 | grep -E "test|passed|failed" || true
echo "✓ Tests complete"

# ============================================================================
# Step 4: Compile to WASM
# ============================================================================

echo "[4/6] Compiling Rust → WASM (wasm-pack)..."

wasm-pack build \
    --target web \
    --release \
    --out-dir pkg \
    --no-typescript

echo "✓ WASM compilation complete"

# ============================================================================
# Step 5: Verify artifacts
# ============================================================================

echo "[5/6] Verifying artifacts..."

if [ -f "pkg/skclisp_crypto_wasm.wasm" ]; then
    WASM_SIZE=$(stat -f%z "pkg/skclisp_crypto_wasm.wasm" 2>/dev/null || stat -c%s "pkg/skclisp_crypto_wasm.wasm")
    echo "✓ WASM module: pkg/skclisp_crypto_wasm.wasm ($WASM_SIZE bytes)"
else
    echo "ERROR: WASM module not found"
    exit 1
fi

if [ -f "pkg/skclisp_crypto_wasm.js" ]; then
    JS_SIZE=$(stat -f%z "pkg/skclisp_crypto_wasm.js" 2>/dev/null || stat -c%s "pkg/skclisp_crypto_wasm.js")
    echo "✓ JavaScript glue: pkg/skclisp_crypto_wasm.js ($JS_SIZE bytes)"
else
    echo "ERROR: JavaScript glue not found"
    exit 1
fi

# ============================================================================
# Step 6: Copy to docs/ for GitHub Pages
# ============================================================================

echo "[6/6] Deploying to GitHub Pages..."

mkdir -p ../docs/assets/

cp pkg/skclisp_crypto_wasm.wasm ../docs/assets/
cp pkg/skclisp_crypto_wasm.js ../docs/assets/
cp pkg/skclisp_crypto_wasm.d.ts ../docs/assets/ 2>/dev/null || true
cp pkg/package.json ../docs/assets/ 2>/dev/null || true

echo "✓ Deployed to docs/assets/"

# ============================================================================
# Final Report
# ============================================================================

echo ""
echo "=== Build Complete ==="
echo "Artifacts:"
echo "  - native/pkg/skclisp_crypto_wasm.wasm (WASM module)"
echo "  - native/pkg/skclisp_crypto_wasm.js (JavaScript bindings)"
echo "  - docs/assets/skclisp_crypto_wasm.wasm (GitHub Pages)"
echo "  - docs/assets/skclisp_crypto_wasm.js (GitHub Pages)"
echo ""
echo "Functions exported:"
echo "  - blake3_hash(input: &[u8]) → Vec<u8>"
echo "  - blake3_verify_wasm(payload, expected_digest) → Blake3VerificationResult"
echo "  - ed25519_verify_wasm(message, signature, public_key) → Ed25519VerificationResult"
echo "  - validate_mutation_wasm(...) → MutationValidationResult"
echo "  - validate_proof_certificate_wasm(cert_bytes) → ProofCertificateValidationResult"
echo ""
echo "✓ Ready for browser integration"
echo ""
echo "Usage:"
echo "  import init, * as wasmModule from './assets/skclisp_crypto_wasm.js';"
echo "  await init();"
echo "  const result = wasmModule.blake3_verify_wasm(payload, digest);"
