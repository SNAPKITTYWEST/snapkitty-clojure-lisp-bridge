# SOVEREIGN KNOWLEDGE ENGINE

<p align="center">
  <strong>Relational LISP execution fabric — McCarthy 1958 to formal proof, WORM-sealed, anti-hallucination oracle gated.</strong>
</p>

<p align="center">
  <a href="https://snapkittywest.github.io/snapkitty-clojure-lisp-bridge/sovereign-runtime.html">
    <img src="./assets/sovereign-runtime-demo.gif" alt="Enter the Sovereign McCarthy LISP Machine" width="100%" />
  </a>
</p>

<p align="center">
  LISP 1958 &rarr; EmojiScript Bytecode &rarr; Relational Proof &rarr; WORM Seal &rarr; Verified Receipt
</p>

---

**Status:** PRODUCTION v2.0.0 - Sprint 2 complete  
**Architecture:** Relational tag engine + 6 integrated systems + anti-hallucination oracle + SNAP OS backend

---

## INTERACTIVE CONSOLES

| Console | What It Does | Link |
|---------|-------------|------|
| Lisp Machine REPL | Evaluate LISP, run EmojiScript, verify crypto | [Open](https://snapkittywest.github.io/snapkitty-clojure-lisp-bridge/sovereign-runtime.html) |
| CRT Terminal | Full xterm.js LISP machine (PWA, offline) | [Open](https://snapkittywest.github.io/snapkitty-clojure-lisp-bridge/lisp-machine-terminal.html) |
| LTMS Console | Assert beliefs, inference rules, semantic search | [Open](https://snapkittywest.github.io/snapkitty-clojure-lisp-bridge/ltms-console.html) |
| VM Debugger | Compile to bytecode, step through execution | [Open](https://snapkittywest.github.io/snapkitty-clojure-lisp-bridge/soulvm-debugger.html) |
| Sovereign Emulator | Full sovereign OS emulator (PWA) | [Open](https://snapkittywest.github.io/snapkitty-clojure-lisp-bridge/sovereign-emulator.html) |

---

## WHAT IS THIS?

A unified LISP execution fabric connecting six sovereign systems into one anti-hallucination pipeline. Every agent claim flows through a relational oracle before it can be sealed to the WORM chain.

**The core insight:** agents do not trust each other. They prove against the relational model. Each tag is a mathematical fact with forward and backward proof obligations. If the proofs do not converge, the claim is rejected.

---

## THE RELATIONAL TAG PIPELINE

```
User writes Clojure in browser
           |
    <compilation>      sourceHash == blake3(source)
           |
    <bytecode>         forward + backward proofs must converge
                       artifactHash == blake3(payload)
           |
    <policy-verdict>   Trust Deed v1.0 gate (EVIDENCE or SILENCE)
           |
    <capability-grant> Silverback mints unforgeable capability
           |
   claimguard oracle   SGML encode -> Z3 hedge check -> exit 0 or 1
           |
    <execution>        resultHash == blake3(result), duration > 0
           |
    <bifrost-seal>     WORM chain, Ed25519 signed, chain continuity
           |
   <verified-receipt>  neuralConfidence: 0.97, tamperingDetected: false
```

Every artifact lives in a markdown file. Tags are the protocol. XSLT rules are the transformation engine. Neural embeddings record constraint state at every transition.

```bash
node backend/relational-engine/pipeline.mjs "(+ 3 4)"
```

---

## SPRINT 2 MODULES ADDED (2026-07-31)

### Relational Engine - backend/relational-engine/

| File | What It Does |
|------|-------------|
| evalo.mjs | miniKanren core (unify/walk/conde/fresh), refinement validators for all 7 tag types, self-correction loop, bidirectional compile (forward + backward synthesis), reflexivity checker, neural bridge recorder, proof certificate generator |
| pipeline.mjs | XSLT-style 7-phase pipeline, EmojiScript VM, claimguard gated execution, markdown artifact renderer |

### SNAP OS Backend - backend/snap-os/ (9 Rust crates cherry-picked from snap-os master)

| Crate | What It Does |
|-------|-------------|
| bifrost/ | WORM chain - Blake3/Ed25519 content-addressed, write-once storage |
| soulvm/ | Cranelift JIT + Immix GC - native x86-64 code generation |
| silverback/ | Capability system - unforgeable cryptographic access control |
| craft-crypto/ | EmojiScript + ScratchBlocks compiler to SoulFunc bytecode |
| soul-bus/ | Inter-agent routing, broadcast, registry |
| soul-agent/ | Agent thread loop |
| soul-narrator/ | Execution narration layer |
| bifrost-policy/ | Lean 4 policy proofs + Prolog governance rules |
| context-hydrator/ | Context enrichment layer |

### SNAP OS Bridge - backend/snap-os-bridge/

| File | What It Does |
|------|-------------|
| jit-gateway.mjs | HTTP bridge: MCP to snap-os JIT to WORM seal (POST /api/snap-os/jit) |
| claimguard.mjs | Anti-hallucination oracle - SGML encode, Z3 structural + hedge check, exit 0 (VERIFIED) or 1 (REJECTED) |

### WASM Crypto - docs/assets/

| File | What It Does |
|------|-------------|
| skclisp_crypto_wasm.js | wasm-bindgen JS glue generated from compiled Rust |
| skclisp_crypto_wasm_bg.wasm | Compiled Rust: Blake3, Ed25519, 8-gate mutation validation, 157-byte proof cert parser |

### BOB Orchestrator - backend/bob/

| File | What It Does |
|------|-------------|
| bob.mjs | Sovereign compliance agent - Trust Deed v1.0 gate, SHA-256 WORM sealing |
| metatron.mjs | METATRON orchestrator |
| shadow-runtime/ | WORM chain, crawlers (ahmad-bot, edualc), Forth interpreter, injector |
| worm/ | WORM S-expressions + meta-repo graveyard (Lisp/Forth per repo) |
| *.deed | Agent trust deeds: ahmad-bot, bob, edualc |

### GitLab Connector - backend/gitlab/

| File | What It Does |
|------|-------------|
| webhook-receiver.mjs | GitLab webhook listener on port 4700 |
| robob-orchestrator.mjs | ROBOB event classifier |
| abzu-bridge.mjs | Phoenix LiveView bridge |
| gitlab-api.mjs | GitLab API wrapper |
| worm-chain.mjs | GitLab WORM chain |

### Governance - governance/

| File | What It Does |
|------|-------------|
| constitution.md | GRISP sovereign constitution |
| trust-deed.md | Trust Deed v1.0 - every compilation gated against this |
| deed-rules.lisp | Trust Deed rules written in LISP |
| worm-chain.mjs | Three-model WORM sealing (Claude + GPT + verification) |
| agents/ | bifrost-translator, icp-verifier, metric-stream, watermark, orchestrate |

### Semantic Passes - backend/semantic-passes.mjs

Four passes run on every bytecode output in sequence:

| Pass | What It Does |
|------|-------------|
| Telemetry | Op count, stack depth, timing, emit to NATS |
| Policy | Trust Deed v1.0 gate - violations produce DENIED verdict |
| Sealing | SHA-256 WORM chain entry, immutable append-only ledger |
| Rights | Score below 0.42 triggers READ_ONLY downgrade + human review required |

### LISP Machines (local) - docs/js/ and backend/lisp-rs/

| File | What It Does |
|------|-------------|
| lisp-machine-legacy.mjs | McCarthy evaluator + safeOps (Apple II Universal Machine) |
| lisp-to-vm.mjs | LISP to VM bytecode (PUSH/ADD/SUB/MUL/DIV/PRINT/HALT) |
| sexpr-parser.mjs | Real S-expression parser |
| lisp-expand.mjs | Macro expansion |
| fontana-decoder.mjs | Fontana FFI decoder |
| fontana-ffi-sim.mjs | Fontana FFI simulator |
| lisp-patterns.mjs | LISP pattern matching engine |
| lisp-machine-terminal.html | Full xterm.js CRT terminal LISP machine (PWA, offline) |
| lisp-machine.tsx | CollectiveKitty Next.js LISP machine page |
| backend/lisp-rs/eval.rs | Rust LISP evaluator (from DEVFLOW-FINANCE/snapkitty-core) |
| backend/lisp-rs/machine.rs | Rust LISP machine |
| backend/lisp-rs/parser.rs | Rust S-expression parser |
| backend/lisp-rs/heap.rs | Rust heap allocator |
| backend/lisp-rs/env.rs | Rust environment and scope |
| backend/lisp-rs/repl.rs | Rust REPL |
| backend/lisp-rs/word.rs | Rust word and symbol types |
| backend/lisp-rs/world.rs | Rust world model |
| backend/lisp-rs/forge.rs | FORGE collision registry - SHA-256, entropy cost, agent pair tracking |
| backend/lisp-rs/forge_engine.rs | FORGE NPC engine |

### DSSSL Synthesis - dsssl-synthesis/

| File | What It Does |
|------|-------------|
| dsssl-synthesis.mjs | Homoiconic SGML grove to S-expr, miniKanren unification, Z3 validation, NaCl receipts |
| dsssl-synthesis-fixed.mjs | Fixed homoiconic DSSSL engine (287 lines) |
| refine-eval-append.mjs | miniKanren + Z3 + Lean4 synthesis pipeline (642 lines) |
| lean/append_certificate.lean | Formal Lean 4 proof |
| INTERLOCK_ARCHITECTURE.md | Tau Prolog + Clojure interlock architecture spec |

---

## ANTI-HALLUCINATION ORACLE

Every agent claim is SGML-encoded and checked before bifrost seals it:

```
Agent output
     |
SGML-encode as <claim>
  <source>...</source><bytecode>...</bytecode>
  <result>...</result><actor>...</actor>
</claim>
     |
Z3 structural + hedge check
  - SGML structure valid?
  - Result contains hedge phrases? (I think, probably, might be, could be)
  - Result non-empty?
     |
Exit 0  ->  bifrost seals as VERIFIED
Exit 1  ->  REJECTED, never enters the chain
```

Agents cannot self-certify. The oracle decides.

---

## WHAT IS REAL (VERIFIED)

| Component | Status | Location |
|-----------|--------|----------|
| Relational engine (miniKanren) | Running | backend/relational-engine/evalo.mjs |
| 7-phase XSLT pipeline | Running | backend/relational-engine/pipeline.mjs |
| claimguard oracle | Running | backend/snap-os-bridge/claimguard.mjs |
| WASM crypto (Blake3, Ed25519) | Compiled | docs/assets/skclisp_crypto_wasm_bg.wasm |
| snap-os Rust crates (9) | Source | backend/snap-os/ |
| Rust LISP machine (8 files) | Source | backend/lisp-rs/ |
| FORGE collision registry | Source | backend/lisp-rs/forge.rs |
| McCarthy evaluator (JS) | Running | docs/js/lisp-machine-legacy.mjs |
| Fontana FFI decoder/simulator | Running | docs/js/fontana-*.mjs |
| CRT terminal LISP machine | Live | docs/lisp-machine-terminal.html |
| BOB orchestrator | Running | backend/bob/bob.mjs |
| GitLab webhook receiver | Running | backend/gitlab/webhook-receiver.mjs |
| GRISP constitution + trust deed | Signed | governance/ |
| Three-model WORM chain | Running | governance/worm-chain.mjs |
| DSSSL synthesis oracle | Running | dsssl-synthesis/dsssl-synthesis.mjs |
| miniKanren synthesis (642 lines) | Running | dsssl-synthesis/refine-eval-append.mjs |
| Lean 4 proofs T01-T04 | Zero sorry | lean-formalization/skclisp/ |
| Coq formal spec (7 modules) | Source | coq/ |
| LTMS (Clojure + Prolog + Haskell) | Running | src/snapkitty/ltms/ |
| 8 MCP tools | Registered | src/snapkitty/lisp/mcp/ |

---

## QUICK START

```bash
# Full relational pipeline - produces 7-tag markdown artifact
node backend/relational-engine/pipeline.mjs "(+ 3 4)"

# With verbose neural embedding trace
node backend/relational-engine/pipeline.mjs "(* 6 7)" --verbose

# Test oracle directly
echo '{"source":"(+ 1 2)","bytecode":"push1 push2 add","result":"3","actor":"user"}' \
  | node backend/snap-os-bridge/claimguard.mjs

# Compile Clojure standalone (no install needed)
node bin/clojure-to-bytecode.mjs myfile.clj

# Start services
npm run serve:bob          # BOB orchestrator
npm run serve:gitlab       # GitLab webhook :4700
npm run serve:snap-os      # SNAP OS JIT bridge :8001
```

---

## DIRECTORY STRUCTURE

```
snapkitty-clojure-lisp-bridge/
|-- backend/
|   |-- relational-engine/   evalo.mjs + pipeline.mjs
|   |-- snap-os/             9 Rust crates
|   |-- snap-os-bridge/      jit-gateway + claimguard oracle
|   |-- bob/                 BOB orchestrator + shadow runtime
|   |-- gitlab/              webhook + ROBOB + ABZU
|   |-- lisp-rs/             Rust LISP machine + FORGE
|   |-- audit/               WozVault browser audit log
|   `-- semantic-passes.mjs
|-- governance/
|   |-- constitution.md + trust-deed.md + deed-rules.lisp
|   |-- worm-chain.mjs
|   `-- agents/
|-- dsssl-synthesis/
|-- docs/
|   |-- assets/              compiled WASM crypto
|   |-- js/                  LISP machines + fontana + patterns
|   |-- lisp-machine-terminal.html
|   |-- sovereign-emulator.html
|   `-- sovereign-runtime.html + ltms-console.html + soulvm-debugger.html
|-- src/snapkitty/lisp/      ClojureScript runtime
|   |-- emojiscript.cljs     15-opcode EmojiScript VM
|   |-- jit.cljs             Cranelift JIT pipeline
|   |-- wasm-bridge.cljs     WASM crypto bindings
|   |-- mcp/                 8 MCP tools
|   `-- ltms/                Clojure + Prolog + Haskell LTMS
|-- native/                  Rust WASM + NASM + C++ binding
|-- lean-formalization/      Lean 4: T01-T04 proven, zero sorry
|-- coq/                     Coq formal spec (7 modules)
|-- orchestrator/shadow/     GRISP shadow arena + Forth interpreter
`-- bin/clojure-to-bytecode.mjs
```

---

## OWNERSHIP

**Built by:** Ahmad Parr (SnapKittyWest) + Claude Code  
**Trust:** Bel Esprit D'Accord Irrevocable Trust  
**License:** Sovereign Source License v1.0  
**Last Updated:** 2026-07-31  
**Status:** Production v2.0.0
