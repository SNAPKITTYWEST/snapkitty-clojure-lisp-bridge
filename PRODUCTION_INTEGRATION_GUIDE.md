# Production Integration Guide: Real Runtimes

**Status**: In Transition  
**Date**: 2026-07-30  
**Target**: Fully real repository runtimes in browser

---

## Architecture

### Current State (Compatibility Phase)

```
Browser (docs/sovereign-runtime.html)
    ↓
docs/js/sovereign-runtime.mjs (JavaScript compatibility bridge)
    ├─ LISP Parser (pure JS, tokenizer + recursive descent)
    ├─ LISP Evaluator (11 built-in functions)
    ├─ EmojiScript/SoulVM (15-opcode stack machine)
    └─ WASM Crypto Bridge (blake3_hash, ed25519_verify_wasm)
```

**Note**: Compatibility bridge is **not** the repository's real ClojureScript runtime. It is a reference implementation with identical API, created while the real build is being provisioned.

### Target State (Real Runtimes)

```
Browser (docs/sovereign-runtime.html)
    ↓
docs/js/main.js (Compiled ClojureScript)
    ├─ snapkitty.lisp.bridge.reader (real LISP parser)
    ├─ snapkitty.lisp.bridge.compiler (real LISP compiler)
    ├─ snapkitty.lisp.emojiscript (real EmojiScript VM)
    ├─ snapkitty.ltms.ltms (real LTMS knowledge system)
    └─ snapkitty.lisp.jit (real SoulVM + proof certificates)
    ↓
docs/wasm/skclisp_crypto_wasm_bg.wasm (real WASM crypto)
```

---

## Real Runtime Locations

### ClojureScript Source

**Compiled from:**
- `src/snapkitty/lisp/bridge/reader.cljs` → LISP tokenizer + parser
- `src/snapkitty/lisp/bridge/compiler.cljs` → Semantic compilation
- `src/snapkitty/lisp/emojiscript.cljs` → EmojiScript bytecode interpreter
- `src/snapkitty/ltms/ltms.cljs` → Layered Truth Maintenance System
- `src/snapkitty/lisp/integration/world.cljs` → Browser entry point

**Build Configuration:**
- `shadow-cljs.edn`: `:browser` target defined
- `deps.edn`: ClojureScript dependencies configured
- `.github/workflows/build-clojurescript.yml`: Production build automation

**Compile Command:**
```bash
npx shadow-cljs release browser
```

**Output:** `docs/js/main.js` (production-optimized)

### LTMS Knowledge Layer

**Real implementation:** `src/snapkitty/ltms/ltms.cljs` (351 lines)

**Features:**
- Fact storage and conflict resolution
- Assumption tracking and dependency management
- Ambiguous concept disambiguation
- Hybrid knowledge (symbolic + embedding fallback)

**Real operations:**
- `(ltms/assert-fact value source confidence)`
- `(ltms/query-fact value)`
- `(ltms/resolve-conflict candidates)`
- `(ltms/mark-outdated fact)`
- `(ltms/inspect-justification fact)`

**Data source:** Pre-seeded from `docs/data/initial-facts.edn` (to be generated)

### Formal Proofs

**Real Lean 4 sources:** `lean-formalization/skclisp/`

**Files:**
- `Machine.lean` — State + execution semantics
- `Mutation.lean` — Mutation journal + rollback
- `Equivalence.lean` — Semantic equivalence proofs
- `Basic.lean` — Core definitions
- `PrimitiveTypes.lean` — Type system
- `ProofCertificate.lean` — Certificate validation

**Build command:**
```bash
cd lean-formalization/skclisp && lake build
```

**Evidence generated:**
- `docs/PROOF_VERIFICATION_REPORT.md` — Build output + metadata
- `docs/data/proofs-evidence.json` — Structured proof status

**Constraints:**
- Zero sorry declarations
- No unresolved goals
- All TypeClass resolution successful
- Termination checking passed

### WASM Cryptography

**Real implementation:** `native/src/lib.rs` (Rust)

**Exports:**
- `blake3_hash(input: &[u8]) -> Vec<u8>`
- `ed25519_verify_wasm(message, signature, public_key) -> bool`
- `validate_mutation_wasm(...) -> MutationValidationResult`
- `validate_proof_certificate_wasm(cert_bytes) -> ProofCertificateValidationResult`

**Build:**
```bash
cd native && wasm-pack build --target web --out-dir ../docs/wasm --release
```

**Output:** `docs/wasm/skclisp_crypto_wasm_bg.wasm` (73 KB)

---

## Integration Phases

### Phase 1: ClojureScript Build (In Progress)

**Status**: Workflow created, pending GitHub Actions execution  
**Trigger**: Push to master or manual workflow dispatch  
**Action**: `.github/workflows/build-clojurescript.yml`

**Steps:**
1. Setup Java + Clojure CLI
2. Install npm dependencies
3. Run `npx shadow-cljs release browser`
4. Generate build report
5. Commit `docs/js/main.js` to master

**Expected output:**
```
docs/js/main.js          (optimized bundle)
docs/js/main.js.map      (source map)
docs/BUILD_REPORT.md     (compiler output + evidence)
```

**Load in browser:**
```html
<script src="/snapkitty-clojure-lisp-bridge/js/main.js"></script>
```

### Phase 2: Lean Proof Verification (In Progress)

**Status**: Workflow created, pending GitHub Actions execution  
**Trigger**: Push to master or manual workflow dispatch  
**Action**: `.github/workflows/verify-lean-proofs.yml`

**Steps:**
1. Setup Lean 4
2. Run `lake build` in `lean-formalization/skclisp/`
3. Extract proof metadata
4. Generate evidence JSON
5. Commit proof artifacts

**Expected output:**
```
docs/PROOF_VERIFICATION_REPORT.md (human-readable)
docs/data/proofs-evidence.json     (structured)
```

**Load in browser:**
```javascript
fetch('/snapkitty-clojure-lisp-bridge/data/proofs-evidence.json')
  .then(r => r.json())
  .then(proofs => {
    // Display exact verification status from real Lean build
  })
```

### Phase 3: LTMS Data Loading (Ready)

**Initial facts:** To be generated from `docs/data/initial-facts.edn`

**Example structure:**
```clojure
[
  {:fact :sky-color :value :blue :source :observation :confidence 0.95 :priority 100}
  {:fact :gravity-direction :value :down :source :physics :confidence 1.0 :priority 200}
]
```

**Load in browser:**
```javascript
fetch('/snapkitty-clojure-lisp-bridge/data/initial-facts.edn')
  .then(r => r.text())
  .then(edn => {
    // Parse and seed LTMS with real facts
    window.snapkitty.ltms.seed(edn);
  })
```

### Phase 4: ONNX Embeddings (Decision Required)

**Current status**: NOT INTEGRATED

**Decision options:**

**Option A: In-Browser ONNX Runtime**
- Port: `ONNX Runtime Web` + real model
- Size: ~50 MB (model) + 5 MB (runtime)
- Latency: 100-500ms per embedding
- Benefit: No server dependency
- Cost: Large download, startup delay

**Option B: Backend Service**
- Reuse existing `MCP` endpoint or create HTTP service
- Size: Minimal client
- Latency: Network + service latency
- Benefit: Fast, cacheable
- Cost: Requires running service

**Option C: Precomputed Index**
- Use static embedding JSON (pre-computed)
- Label as: "Precomputed Semantic Index (not live inference)"
- Size: ~1-5 MB
- Latency: Instant (in-memory)
- Benefit: Fast, no dependencies
- Cost: Static data; no real-time embeddings

**Recommendation for this release**: **Option C** (precomputed index) with path forward to Option B (backend service). This allows knowledge queries to work while maintaining honesty about live inference availability.

---

## Real Runtime API

Once ClojureScript compiles, the browser will have access to:

```javascript
// LISP operations
window.snapkitty.lisp.bridge.parseLisp(source: string) -> AST
window.snapkitty.lisp.bridge.compileLisp(ast: AST) -> CompiledForm
window.snapkitty.lisp.evaluateLisp(source: string) -> Result

// EmojiScript/SoulVM
window.snapkitty.lisp.emojiscript.compile(source: string) -> Bytecode
window.snapkitty.lisp.emojiscript.execute(bytecode: Bytecode) -> ExecutionResult

// LTMS Knowledge
window.snapkitty.ltms.assertFact(value, source, confidence) -> void
window.snapkitty.ltms.queryFact(value) -> FactResult
window.snapkitty.ltms.resolvConflict(candidates) -> Fact
window.snapkitty.ltms.inspectJustification(fact) -> Justification

// WASM Crypto
window.skclisp_crypto_wasm.blake3_hash(input: Uint8Array) -> Uint8Array
window.skclisp_crypto_wasm.ed25519_verify_wasm(msg, sig, pk) -> boolean
```

---

## Compatibility Bridge (Temporary)

**File**: `docs/js/sovereign-runtime.mjs`

**Status**: Production fallback while real build is pending

**API Surface**: Identical to real runtime (above)

**Differences**:
- JavaScript implementation (not ClojureScript)
- Simplified LTMS (in-memory only, no persistence)
- No formal proof verification
- No JIT compilation
- No MCP integration

**Removal**: Delete once ClojureScript build is confirmed working

---

## Deployment Checklist

- [ ] GitHub Actions runners provisioned with Java + Clojure CLI
- [ ] ClojureScript build completes successfully (check Actions tab)
- [ ] `docs/js/main.js` generated and committed
- [ ] Lean proofs verify without errors (check Actions tab)
- [ ] `docs/data/proofs-evidence.json` generated
- [ ] LTMS fact seeding configured
- [ ] ONNX decision made (Option A/B/C)
- [ ] UI updated to use real runtime APIs
- [ ] Production URL tested: `/snapkitty-clojure-lisp-bridge/sovereign-runtime.html`
- [ ] All controls verified against real implementations

---

## Next Actions

1. **Trigger ClojureScript build:**
   - Go to GitHub repo → Actions → "Build ClojureScript Runtime" → Run workflow
   - Wait for completion (5-10 minutes)

2. **Trigger Lean verification:**
   - Go to GitHub repo → Actions → "Verify Lean 4 Proofs" → Run workflow
   - Wait for completion (2-5 minutes)

3. **Update HTML to load real runtime:**
   - Once `docs/js/main.js` exists, update script tag
   - Test in browser developer console

4. **Seed LTMS facts:**
   - Create `docs/data/initial-facts.edn`
   - Populate with real knowledge

5. **Make ONNX decision:**
   - Choose Option A/B/C
   - Implement accordingly

6. **Smoke test production:**
   - Evaluate `(+ 1 2)` → 3
   - Execute `🔢6 🔢7 ✖️ ↩️` → 42
   - Query knowledge base
   - Verify proofs display real evidence

---

**Owner**: Jessica (SnapKittyWest)  
**Last Updated**: 2026-07-30  
**Maintenance**: Update as workflows complete and real runtimes integrate
