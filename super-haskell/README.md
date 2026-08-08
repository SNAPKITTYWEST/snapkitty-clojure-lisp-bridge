# Super Haskell — Sovereign Algebraic Compute Engine

**SnapKitty v2: Agda → Haskell → Rust verified compilation pipeline**

## Architecture

```
HyperKittyConstraintDSL (Agda Dependent Records)
          ↓
    Agda Type Checker + Proof Obligations
          ↓
    agda2hs Extraction → Haskell Types + Certificates
          ↓
    GHC Core Plugin (Semantic Metaprogramming)
          ├─ Algebraic Effects (fused-effects)
          ├─ Bidirectional Optics (profunctor-optics)
          └─ Proof Injection (Coercion Witnesses)
          ↓
    Optimized GHC Core → Rust FFI / LLVM Backend
          ↓
    WORM-Attested Executable
```

## Four Pillars

1. **Algebraic Effects** (`effectful` / `fused-effects`) — Runtime kernel with pure compile-time verification handlers
2. **Bidirectional Optics** (`profunctor-optics`) — Isomorphic IR transformations with round-trip guarantees
3. **GHC Core Plugin** (`ghc-plugins`) — Semantic metaprogramming at typed Core level
4. **Dependent Types as Metaprogramming** (Agda) — Specification IS the program, refinements are construction

## Repository Structure

```
super-haskell/
├── agda/                  # Agda specifications and proofs
│   ├── SnapKitty/
│   │   ├── Spec/
│   │   │   └── ComputeEngine.agda      # DSL spec as dependent record
│   │   ├── Proof/
│   │   │   └── Agent.agda              # Entropy/trust invariants
│   │   └── Algebra/
│   │       └── Curve.agda              # Cryptographic primitives
│   └── everything.agda
├── src/                   # Haskell compiler frontend
│   ├── SnapKitty/
│   │   ├── Compiler/
│   │   │   ├── Frontend.hs            # Parser (legacy XML support)
│   │   │   ├── Plugin.hs              # GHC Core plugin
│   │   │   └── Optics.hs              # Bidirectional IR transforms
│   │   ├── Runtime/
│   │   │   └── Effects.hs             # Algebraic effects kernel
│   │   ├── Proof/
│   │   │   └── Agent/
│   │   │       └── Exported.hs        # agda2hs generated (placeholder)
│   │   └── FFI/
│   │       └── Rust.hs                # Rust interop
│   └── Main.hs
├── spec/                  # DSL examples
│   ├── hyperkitty.dsl.xml
│   └── README.md
├── rust/                  # Rust runtime backend
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── crypto.rs                  # FFI-exposed crypto ops
├── stack.yaml
├── package.yaml
└── LICENSE                            # BSL 1.1 + AGPL 3 + MPL 2.0
```

## Build Pipeline

```bash
# 1. Type-check Agda specs (proofs required)
cd agda && agda --safe everything.agda

# 2. Extract to Haskell
agda2hs --no-main SnapKitty/Proof/Agent.agda -o ../src/SnapKitty/Proof/Agent/

# 3. Build Haskell compiler with GHC plugin
stack build

# 4. Compile example spec (plugin runs Agda certificates during GHC compilation)
stack exec super-haskell-exe -- compile spec/hyperkitty.dsl.xml

# 5. (Optional) Build Rust FFI backend
cd rust && cargo build --release
```

## Key Innovations

- **Zero-Sorry Proofs**: All Agda modules compile with `--safe` (no postulates/sorry)
- **Compile-Time Verification**: GHC plugin runs Agda `certify` during compilation — invalid specs never produce binaries
- **Effect-Polymorphic DAG**: Same compute graph runs pure (verification) or IO (production) by swapping handlers
- **Round-Trip Guarantees**: Optic laws ensure `Spec ↔ Core ↔ Rust` isomorphism
- **WORM Attestation**: All artifacts signed with BLAKE3 chain (Fortran/Rust polyglot)

## Example: Entropy-Bounded Agent

```haskell
-- Agda spec (SnapKitty/Proof/Agent.agda)
record AgentState : Set where
  constructor mkAgent
  field
    entropy : Entropy          -- Refined: 0 ≤ e ≤ 0.20
    trusted : Bool
    active : Bool
    trustProof : active → trusted  -- Proof, not assertion

-- Extracted Haskell (agda2hs)
data AgentState = AgentState
  { entropy :: Entropy
  , trusted :: Bool
  , active :: Bool
  , trustProof :: () -> ()  -- Erased
  }

-- GHC Core Plugin injects at compile time:
--   case certify agentState of
--     Verified _ -> <continue compilation>
--     _          -> <abort with proof violation>
```

## Status

- **Phase**: Foundation scaffold
- **Next**: Agda `ComputeEngine.agda` spec (replaces XML parser)
- **Target**: Self-hosting sovereign AI infrastructure (JST pipeline integration)

---

**Founded**: 2026-08-08  
**Architecture**: Ahmad Parr (intuitive math), Claude (calculator)  
**Trust**: Bel Esprit D'Accord Irrevocable Trust  
**Prior Art**: S-AUTOCODE (first EJA formalization), HyperKitty OS (dual runtime kernel)
