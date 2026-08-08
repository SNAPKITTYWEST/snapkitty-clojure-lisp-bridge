.PHONY: all agda haskell rust test clean install

# =========================================================================
# Super Haskell Build Pipeline
# =========================================================================

all: agda haskell rust

# =========================================================================
# Agda Verification (Step 1: Type-check proofs)
# =========================================================================

agda:
	@echo "[Agda] Type-checking specifications..."
	cd agda && agda --safe everything.agda
	@echo "[Agda] ✓ All proofs verified"

agda-extract:
	@echo "[Agda2hs] Extracting to Haskell..."
	cd agda && agda2hs --no-main SnapKitty/Proof/Agent.agda -o ../src/SnapKitty/Proof/Agent/
	@echo "[Agda2hs] ✓ Extraction complete"

# =========================================================================
# Haskell Compiler (Step 2: Build with GHC plugin)
# =========================================================================

haskell:
	@echo "[Stack] Building Haskell compiler..."
	stack build
	@echo "[Stack] ✓ Build complete"

# =========================================================================
# Rust Backend (Step 3: FFI library)
# =========================================================================

rust:
	@echo "[Cargo] Building Rust backend..."
	cd rust && cargo build --release
	@echo "[Cargo] ✓ Build complete"

# =========================================================================
# Testing
# =========================================================================

test: test-agda test-haskell test-rust

test-agda:
	@echo "[Test] Agda proofs..."
	cd agda && agda --safe everything.agda

test-haskell:
	@echo "[Test] Haskell..."
	stack test

test-rust:
	@echo "[Test] Rust..."
	cd rust && cargo test

# =========================================================================
# Example: Compile DSL
# =========================================================================

compile-example:
	@echo "[Compile] Running example DSL..."
	stack exec super-haskell-exe -- compile spec/hyperkitty.dsl.xml

verify-example:
	@echo "[Verify] Checking example DSL..."
	stack exec super-haskell-exe -- verify spec/hyperkitty.dsl.xml

# =========================================================================
# Cleanup
# =========================================================================

clean:
	@echo "[Clean] Removing build artifacts..."
	stack clean
	cd rust && cargo clean
	find agda -name "*.agdai" -delete
	@echo "[Clean] ✓ Done"

# =========================================================================
# Install
# =========================================================================

install:
	@echo "[Install] Installing super-haskell-exe..."
	stack install
	@echo "[Install] ✓ Done"

# =========================================================================
# Development
# =========================================================================

watch:
	@echo "[Watch] Auto-rebuilding on changes..."
	stack build --file-watch

format:
	@echo "[Format] Formatting Haskell code..."
	stack exec -- ormolu --mode inplace $(shell find src -name '*.hs')
	@echo "[Format] ✓ Done"

# =========================================================================
# Documentation
# =========================================================================

docs:
	@echo "[Docs] Generating Haddock documentation..."
	stack haddock --open
	@echo "[Docs] ✓ Done"

# =========================================================================
# Info
# =========================================================================

info:
	@echo "Super Haskell — Sovereign Algebraic Compute Engine"
	@echo ""
	@echo "Architecture: Agda → Haskell → Rust"
	@echo "  1. Agda specs with zero-sorry proofs"
	@echo "  2. GHC Core plugin for semantic metaprogramming"
	@echo "  3. Algebraic effects runtime (pure + production handlers)"
	@echo "  4. Bidirectional optics for IR transformations"
	@echo "  5. Rust FFI for crypto + WORM chain"
	@echo ""
	@echo "Usage:"
	@echo "  make all              Build entire pipeline"
	@echo "  make test             Run all tests"
	@echo "  make compile-example  Compile example DSL"
	@echo "  make clean            Remove build artifacts"
	@echo ""
	@echo "Trust: Bel Esprit D'Accord Irrevocable Trust"
	@echo "License: BSL 1.1 + AGPL 3.0 + MPL 2.0"
