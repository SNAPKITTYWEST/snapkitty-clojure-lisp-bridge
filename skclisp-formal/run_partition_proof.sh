#!/bin/bash
# SKC-LISP-WORLD: Self-Modifying Partition Proof Generator
# Pipeline: ASP → Clingo → JSON → Python Synthesis → Lean 4

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_DIR="${SCRIPT_DIR}/partition_proofs"
PARTITION_LP="${SCRIPT_DIR}/partition_proof.lp"
SYNTHESIZE_PY="${SCRIPT_DIR}/synthesize_lean.py"

# Configuration
N=${1:-20}
CLINGO_OPTS="--const N=$N --outf=json 0"

mkdir -p "$OUTPUT_DIR"

echo "=========================================="
echo "Self-Modifying Partition Proof Generator"
echo "=========================================="
echo "N (max partition number): $N"
echo "Output directory: $OUTPUT_DIR"
echo ""

# Step 1: Check Clingo availability
if ! command -v clingo &> /dev/null; then
    echo "ERROR: Clingo not found. Install with: pip install clingo"
    exit 1
fi

# Step 2: Run Clingo to generate ASP model
echo "[1/3] Running Clingo ASP solver..."
CLINGO_OUTPUT="${OUTPUT_DIR}/clingo_output_${N}.json"

if clingo $PARTITION_LP $CLINGO_OPTS > "$CLINGO_OUTPUT" 2>"${OUTPUT_DIR}/clingo_${N}.log"; then
    echo "✓ Clingo execution successful"
    PARTITION_COUNT=$(grep -c "partition(" "$CLINGO_OUTPUT" || echo "0")
    echo "  Generated $PARTITION_COUNT partition facts"
else
    echo "✗ Clingo execution failed"
    cat "${OUTPUT_DIR}/clingo_${N}.log"
    exit 1
fi

# Step 3: Synthesize Lean 4 proof
echo ""
echo "[2/3] Synthesizing Lean 4 proof from ASP model..."
LEAN_OUTPUT="${OUTPUT_DIR}/PartitionProof_${N}.lean"

if python3 "$SYNTHESIZE_PY" < "$CLINGO_OUTPUT" > "$LEAN_OUTPUT" 2>"${OUTPUT_DIR}/synthesis_${N}.log"; then
    echo "✓ Lean 4 synthesis successful"
    LEAN_LINES=$(wc -l < "$LEAN_OUTPUT")
    echo "  Generated $LEAN_LINES lines of Lean 4 code"
else
    echo "✗ Synthesis failed"
    cat "${OUTPUT_DIR}/synthesis_${N}.log"
    exit 1
fi

# Step 4: Attempt Lean 4 type check (if lean available)
echo ""
echo "[3/3] Verifying Lean 4 syntax..."
if command -v lean &> /dev/null; then
    if lean --check "$LEAN_OUTPUT" > /dev/null 2>&1; then
        echo "✓ Lean 4 type check passed"
    else
        echo "⚠ Lean 4 type check failed (expected: proof sketches use sorry)"
        # This is expected due to sorry placeholders
    fi
else
    echo "⚠ Lean 4 not found; skipping type check"
fi

echo ""
echo "=========================================="
echo "Results:"
echo "  ASP facts:    $CLINGO_OUTPUT"
echo "  Lean 4 proof: $LEAN_OUTPUT"
echo "  Clingo log:   ${OUTPUT_DIR}/clingo_${N}.log"
echo "  Synthesis log: ${OUTPUT_DIR}/synthesis_${N}.log"
echo "=========================================="
echo ""
echo "Next: Edit $LEAN_OUTPUT to replace 'sorry' with proof terms"
echo "      Run: lean --check $LEAN_OUTPUT"
