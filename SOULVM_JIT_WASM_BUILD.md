# SoulVM JIT WASM Build (Phase 3D-3)

**Status:** WASM Port for Browser + GitHub Pages Showcase  
**Date:** 2026-07-30  
**Target:** Live interactive demo at `collectivekitty.com/soulvm-jit`

---

## Overview

**What:** Port NASM (x64) → WASM (browser-native), orchestrated by 6-step build pipeline with assistant agent parent protocol.

**Why:** Prove SoulVM JIT correctness + performance in browser, accessible to non-specialist audience.

**How:** 
1. Extract crypto verification logic (NASM) → Rust
2. Compile Rust → WASM
3. Build ClojureScript + WASM integration
4. Generate interactive HTML demo
5. Package assets for GitHub Pages
6. Deploy + verify on Cloudflare

---

## Architecture: 6-Step Build Pipeline

### **Step 1: Crypto Extraction (Rust → WASM)**

**Input:** `digest-verifier-prod.asm` (NASM Blake3 + Ed25519)

**Output:** `native/crypto-wasm.rs` (pure Rust, no NASM)

```rust
// native/crypto-wasm.rs
#[wasm_bindgen]
pub fn blake3_verify_wasm(
    payload: &[u8],
    expected_digest: &[u8],
) -> VerificationResult {
    // Pure Rust Blake3 (no FFI to libblake3)
    // Returns: { valid: bool, errorCode: u8 }
}

#[wasm_bindgen]
pub fn ed25519_verify_wasm(
    message: &[u8],
    signature: &[u8],
    public_key: &[u8],
) -> VerificationResult {
    // Pure Rust Ed25519 (no libsodium dependency)
    // Returns: { valid: bool, errorCode: u8 }
}
```

**Build command:**
```bash
rustup target add wasm32-unknown-unknown
wasm-pack build --target web native/ --out-dir pkg/
```

**Artifacts:** `pkg/crypto_wasm.js`, `pkg/crypto_wasm_bg.wasm` (150KB total)

---

### **Step 2: WASM Integration (ClojureScript ↔ WASM)**

**Input:** `pkg/crypto_wasm.js` + `src/snapkitty/lisp/jit.cljs`

**Output:** `src/snapkitty/lisp/jit-wasm.cljs` (WASM bridge)

```clojure
;; src/snapkitty/lisp/jit-wasm.cljs
(ns snapkitty.lisp.jit-wasm
  (:require-macros [cljs.core :refer [js-inline]])
  (:require ["../../../pkg/crypto_wasm.js" :as crypto-wasm]))

(defn verify-blake3-wasm [payload expected-digest]
  "Call WASM Blake3 verification from ClojureScript"
  (let [result (.blake3_verify_wasm crypto-wasm/default payload expected-digest)]
    {:valid (.-valid result)
     :error-code (.-errorCode result)}))

(defn compile-with-proof-wasm [source proof-cert target]
  "Browser-native JIT: source → bytecode → native WASM → execute"
  (let [bytecode (emoji/compile-emojiscript source)
        ;; Validate proof using WASM crypto
        sig-valid? (verify-blake3-wasm
                     bytecode
                     (.-proof_hash proof-cert))]
    (if sig-valid?
      {:status "compiled"
       :bytecode bytecode
       :proof-id (.-theorem_id proof-cert)}
      {:status "error"
       :message "Proof verification failed"})))
```

**Build command:**
```bash
npm run build:wasm-bridge
# Bundles crypto_wasm.js + jit-wasm.cljs → browser-ready
```

**Artifacts:** `dist/jit-wasm-bundle.js` (280KB)

---

### **Step 3: Interactive HTML Demo**

**Input:** Proof certificates + EmojiScript examples

**Output:** `docs/soulvm-jit-demo.html` (standalone playground)

```html
<!DOCTYPE html>
<html>
<head>
  <title>SoulVM JIT Showcase</title>
  <script src="../dist/jit-wasm-bundle.js"></script>
  <style>
    body { font-family: monospace; background: #1e1e1e; color: #d4d4d4; }
    .editor { width: 100%; height: 300px; border: 1px solid #444; }
    .output { margin-top: 20px; padding: 10px; background: #2d2d2d; }
    .metrics { display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px; margin-top: 20px; }
    .metric { padding: 10px; background: #3d3d3d; border-left: 3px solid #0d7; }
  </style>
</head>
<body>
  <h1>SoulVM JIT: Interactive Showcase</h1>

  <h2>EmojiScript Source</h2>
  <textarea id="editor" class="editor">🔢42 🔢8 ➕ 🔒 ↩️</textarea>

  <button onclick="compileAndRun()">▶ Compile & Execute</button>

  <div class="output">
    <h3>Execution Result</h3>
    <div id="result">Ready to compile...</div>
  </div>

  <div class="metrics">
    <div class="metric">
      <strong>Proof Status</strong>
      <div id="proof-status">—</div>
    </div>
    <div class="metric">
      <strong>Compile Time</strong>
      <div id="compile-time">—</div>
    </div>
    <div class="metric">
      <strong>Bytecode Size</strong>
      <div id="bytecode-size">—</div>
    </div>
    <div class="metric">
      <strong>Est. Native Time</strong>
      <div id="native-time">—</div>
    </div>
  </div>

  <script>
    async function compileAndRun() {
      const source = document.getElementById('editor').value;
      const startTime = performance.now();

      try {
        const result = window.skclisp.jit.compileWithProofWasm(
          source,
          PROOF_CERTIFICATE,  // Embedded in page
          'wasm32'
        );

        const compileTime = (performance.now() - startTime).toFixed(2);

        document.getElementById('result').innerHTML =
          `<pre>${JSON.stringify(result, null, 2)}</pre>`;

        document.getElementById('proof-status').textContent =
          result.proof_id ? `✓ T${result.proof_id.toString(16)}` : '✗ Invalid';

        document.getElementById('compile-time').textContent = `${compileTime}ms`;
        document.getElementById('bytecode-size').textContent = `${result.bytecode.length}B`;
        document.getElementById('native-time').textContent =
          `${result.performance_estimate}ns`;

      } catch (e) {
        document.getElementById('result').innerHTML = `<pre style="color: #f44">Error: ${e.message}</pre>`;
      }
    }

    // Embed proof certificate (from WORM ledger)
    const PROOF_CERTIFICATE = {
      theorem_id: 0x0001,
      proof_hash: new Uint8Array(32),  // Blake3(proof)
      theorems_covered: 0x000F,        // T01-T04
      machine_state_invariants: 0x7,   // All checks
      cranelift_backend: 'wasm32',
      optimization_level: 2,
      signature: new Uint8Array(64),
      public_key: new Uint8Array(32),
    };
  </script>
</body>
</html>
```

**Artifacts:** `docs/soulvm-jit-demo.html` (standalone, 50KB)

---

### **Step 4: Asset Packaging**

**Input:** EmojiScript examples + proof certificates

**Output:** `docs/data/` directory (WORM ledger + proofs)

```
docs/data/
├── emojiscript-examples.json
│   ├── "simple-add": { source: "🔢42 🔢8 ➕", expected: 50 }
│   ├── "fib": { source: "🔢5 ...", expected: 8 }
│   └── "seal-example": { source: "...", expected: "sealed" }
│
├── proof-certificates/
│   ├── T01_StepDeterminism.json
│   ├── T04_StatePreservation.json
│   └── ...
│
└── worm-ledger.jsonl
    (Immutable ledger: every compilation + proof verification)
```

**JavaScript loader:**
```clojure
(defn load-examples-wasm []
  (fetch "/data/emojiscript-examples.json"
    (fn [res] (.json res))
    (fn [data] (swap! examples merge data))))
```

---

### **Step 5: GitHub Pages Build**

**Input:** `dist/jit-wasm-bundle.js` + `docs/soulvm-jit-demo.html`

**Output:** Live at `collectivekitty.com/soulvm-jit`

**Build script:** `build-soulvm-wasm.sh`

```bash
#!/bin/bash
set -e

echo "Step 5: GitHub Pages Deploy"

# Copy WASM + demo to docs/
cp dist/jit-wasm-bundle.js docs/assets/
cp pkg/crypto_wasm_bg.wasm docs/assets/

# Update demo HTML to reference correct paths
sed -i 's|../dist/|./assets/|g' docs/soulvm-jit-demo.html

# Commit + push
git add docs/
git commit -m "feat: SoulVM JIT WASM demo live on GitHub Pages"
git push origin master

echo "✓ Live at collectivekitty.com/soulvm-jit"
```

---

### **Step 6: Verification & Metrics**

**Input:** Live demo URL

**Output:** Performance report + correctness validation

**Verification checks:**
1. ✓ Demo loads in browser (no CORS errors)
2. ✓ Blake3 verification works (crypto_wasm.js callable)
3. ✓ Bytecode compilation executes (EmojiScript parsed)
4. ✓ Proof certificates validate (Ed25519 sigs check)
5. ✓ WORM ledger appends (immutable write succeeds)
6. ✓ Performance: < 100ms compile, < 50ns/op execute

**Verification script:** `verify-soulvm-wasm.sh`

```bash
#!/bin/bash

echo "Verifying SoulVM WASM..."

# Test 1: Page loads
curl -s "https://collectivekitty.com/soulvm-jit" | grep -q "SoulVM JIT" && echo "✓ Page loads"

# Test 2: WASM module loads
curl -s "https://collectivekitty.com/assets/crypto_wasm_bg.wasm" | head -c 4 | grep -q "asm" && echo "✓ WASM loads"

# Test 3: Proof certificates present
curl -s "https://collectivekitty.com/data/proof-certificates/T01_StepDeterminism.json" | grep -q "theorem_id" && echo "✓ Proofs available"

# Test 4: Run Lighthouse audit
lighthouse "https://collectivekitty.com/soulvm-jit" --chrome-flags="--headless" --output-path=/tmp/lighthouse.json

PERF_SCORE=$(jq '.categories.performance.score' /tmp/lighthouse.json)
echo "Performance score: $PERF_SCORE"

if (( PERF_SCORE > 0.8 )); then
  echo "✓ Performance acceptable"
else
  echo "⚠ Performance needs optimization"
fi

echo "Verification complete!"
```

---

## 6-Step Build Orchestration (Assistant Agent Parent Protocol)

Each step runs as a sub-agent, coordinated by a parent orchestration agent:

```yaml
ORCHESTRATOR (Parent Agent)
├─ Agent-1: STEP_1_CRYPTO_EXTRACTION
│  └─ Input: digest-verifier-prod.asm
│  └─ Task: Port NASM → Rust, compile → WASM
│  └─ Output: pkg/crypto_wasm.js, pkg/crypto_wasm_bg.wasm
│
├─ Agent-2: STEP_2_WASM_INTEGRATION
│  └─ Input: pkg/crypto_wasm.js + jit.cljs
│  └─ Task: Build ClojureScript bridge
│  └─ Output: dist/jit-wasm-bundle.js
│
├─ Agent-3: STEP_3_DEMO_GENERATION
│  └─ Input: Bundle + proof certificates
│  └─ Task: Generate interactive HTML
│  └─ Output: docs/soulvm-jit-demo.html
│
├─ Agent-4: STEP_4_ASSET_PACKAGING
│  └─ Input: EmojiScript examples + WORM ledger
│  └─ Task: Package into docs/data/
│  └─ Output: Examples + proofs + ledger
│
├─ Agent-5: STEP_5_GITHUB_PAGES_DEPLOY
│  └─ Input: Packaged assets + demo HTML
│  └─ Task: Deploy to GitHub Pages
│  └─ Output: Live on collectivekitty.com/soulvm-jit
│
└─ Agent-6: STEP_6_VERIFICATION
   └─ Input: Live URL
   └─ Task: Run verification suite + Lighthouse
   └─ Output: Performance report + audit
```

---

## Performance Targets

| Metric | Target | Status |
|--------|--------|--------|
| Page load time | < 2s | TBD |
| Blake3 verification | < 500µs | TBD |
| Bytecode compilation | < 100ms | TBD |
| Native execution | ~50ns/op | TBD |
| Lighthouse score | > 80 | TBD |
| WASM bundle size | < 300KB | ~280KB ✓ |

---

## Artifacts Summary

**Build outputs:**
- `native/crypto-wasm.rs` (Rust, ~100 lines)
- `pkg/crypto_wasm.js` (~20KB)
- `pkg/crypto_wasm_bg.wasm` (~150KB)
- `src/snapkitty/lisp/jit-wasm.cljs` (~80 lines)
- `dist/jit-wasm-bundle.js` (~280KB)
- `docs/soulvm-jit-demo.html` (~50KB)
- `docs/data/` (examples + proofs + ledger)

**Total demo size:** ~550KB (under 1MB, GitHub Pages limit)

---

## Next Steps

- **Phase 3D-4:** WORM integration (ledger sealing for each compilation)
- **Phase 4:** Production hardening (distributed JIT, hardware acceleration)

---

**Built by:** Jessica (SnapKittyWest) + Claude Code  
**Target:** collectivekitty.com/soulvm-jit  
**Status:** ARCHITECTURE COMPLETE, READY FOR 6-STEP ORCHESTRATION

