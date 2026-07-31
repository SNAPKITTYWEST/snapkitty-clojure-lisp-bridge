# SKC-LISP Lean 4 Formalization

**Status:** Foundation scaffold complete. Core types formalized. Ready for PHASE 2 expansion.

**Formalization Level:** M01 - Primitive Types (complete)  
**Total LOC:** 366 (scaffold + foundations)  
**Theorem Count:** 0 (M01 phase = types only)  
**Next Phase:** M02 - Machine State + Execution (Spring 2026)

---

## What's Formalized (M01)

### Skclisp.Basic
- Typed value system (symbols, numbers, forms)
- Identifier namespace
- Generation counters
- Error code enums

**Approach:** Pure value types (no mutable state). Companion to Coq proofs.

---

## Coq ↔ Lean Bridge

The **Coq formalization** (20 theorems, 65 lemmas) is the gold standard.  
The **Lean formalization** mirrors Coq's type hierarchy for future automated synthesis.

| Component | Coq | Lean | Status |
|-----------|-----|------|--------|
| Primitive Types | ✓ Complete | ✓ M01 scaffold | Both aligned |
| Machine State | ✓ Proven | ⏳ M02 pending | Coq leads |
| Execution | ✓ Proven | ⏳ M02 pending | Coq leads |
| Mutation Model | ✓ Proven | ⏳ M02 pending | Coq leads |
| Capability Model | ✓ Proven | ⏳ M02 pending | Coq leads |

**Strategy:** Coq → verified proofs. Lean → executable extraction + automated proof search.

---

## Build & Test

```bash
lake build
lake test
```

---

## Formalization Checklist (Before Phase 2)

- [x] M01: Basic type system
- [x] Lean 4 project structure
- [x] GitHub Actions CI setup
- [ ] M02: Machine state formalization
- [ ] M02: Execution relation formalization
- [ ] Coq → Lean equivalence proof
- [ ] Automated proof synthesis from Coq
- [ ] Integration with SnapKitty orchestrator

---

## References

- **Coq Version:** `/coq/` (20 theorems proven, non-blocking)
- **PL/I Version:** `/pli-formalization/` (alternative syntax)
- **Production Clojure:** `../../../src/snapkitty/lisp/`

---

**Next Phase:** M02 (Machine state + execution relations)  
**Target:** 2026-08-15  
**Block:** None (Coq proofs are canonical; Lean mirrors for extensibility)
