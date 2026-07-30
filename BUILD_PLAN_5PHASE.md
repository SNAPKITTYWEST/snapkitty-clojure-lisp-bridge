# SNAPKITTY CLOJURE LISP BRIDGE — 5-PHASE BUILD PLAN

**Status:** AWAITING XML PROMPT  
**Target Model:** Claude Opus 5 (Sonnet)  
**Architecture:** Auto-addressing per phase

---

## PHASE STRUCTURE

Each phase has:
- **Phase ID** — `PH1`, `PH2`, `PH3`, `PH4`, `PH5`
- **Step ID** — `PH#.S#` (e.g., PH1.S1, PH1.S2)
- **Deliverables** — Specific output artifacts
- **Verification** — Pass/fail criteria

---

## PHASE 1: FOUNDATION & DIALECT REGISTRATION

**Phase ID:** `PH1`  
**Duration:** ~2-3 hours  
**Objective:** Register all LISP world sources, establish unified address space

### Steps

| ID | Step | Deliverable | Verification |
|---|---|---|---|
| PH1.S1 | Discover lisp-machine repo structure | Manifest of McCarthy-1958 dialect | README.md + 5+ source files found |
| PH1.S2 | Discover apple-ii-universal-machine | Manifest of AppleSoft LISP dialect | Struct map + code samples |
| PH1.S3 | Register both dialects in world | `world-registry` with both entries | `(list-world-sources)` returns 2 |
| PH1.S4 | Map LISP code paths to embeddings | Address map (dialect → embedding dim) | JSON manifest created |
| PH1.S5 | Create inter-dialect glossary | Symbol → address translation table | CSV/JSON file with 50+ entries |

**Output:** `PH1_MANIFEST.json`, `PH1_DIALECT_REGISTRY.edn`, `PH1_ADDRESS_MAP.json`

---

## PHASE 2: CODE INGESTION & COMPILATION

**Phase ID:** `PH2`  
**Duration:** ~3-4 hours  
**Objective:** Read, parse, compile all LISP code; generate initial embeddings

### Steps

| ID | Step | Deliverable | Verification |
|---|---|---|---|
| PH2.S1 | Batch ingest McCarthy-1958 LISP | 100+ forms compiled | No parse errors in log |
| PH2.S2 | Batch ingest AppleSoft LISP | 50+ forms compiled | Dialect-specific syntax handled |
| PH2.S3 | Generate embeddings for all forms | Vector store populated (Qdrant) | `count(vectors) >= 150` |
| PH2.S4 | Index symbol cross-references | Graph of symbol → usage locations | Graph file + statistics |
| PH2.S5 | Validate rate limiting | No ingestion throttles hit | Token bucket never depleted |

**Output:** `PH2_INGESTION_REPORT.json`, `PH2_EMBEDDING_STATS.json`, `PH2_SYMBOL_GRAPH.gml`

---

## PHASE 3: SEMANTIC BRIDGE & CROSS-DIALECT LINKING

**Phase ID:** `PH3`  
**Duration:** ~2-3 hours  
**Objective:** Link semantically equivalent forms across dialects

### Steps

| ID | Step | Deliverable | Verification |
|---|---|---|---|
| PH3.S1 | Find semantic duplicates (cosine > 0.95) | List of cross-dialect matches | 20+ high-confidence pairs |
| PH3.S2 | Create equivalence classes | Symbol clusters (McCarthy ≈ AppleSoft) | Cluster file + statistics |
| PH3.S3 | Generate translation mappings | Dialect A → Dialect B form converter | Mapping table (100+ rules) |
| PH3.S4 | Test round-trip translation | Form → A → B → A consistency | 95%+ fidelity on sample |
| PH3.S5 | Build unified symbol namespace | Single namespace with origin tags | RDF/Turtle export |

**Output:** `PH3_SEMANTIC_MATCHES.json`, `PH3_TRANSLATION_RULES.edn`, `PH3_UNIFIED_NAMESPACE.ttl`

---

## PHASE 4: MCP TOOLING & INTEGRATION ENDPOINTS

**Phase ID:** `PH4`  
**Duration:** ~3-4 hours  
**Objective:** Expose all LISP bridges via MCP tools; wire integration points

### Steps

| ID | Step | Deliverable | Verification |
|---|---|---|---|
| PH4.S1 | Create `query_lisp` MCP tool | Query across all dialects | Tool accepts Zod schema |
| PH4.S2 | Create `translate_form` MCP tool | Convert form between dialects | 10+ test cases passing |
| PH4.S3 | Create `find_equivalent` MCP tool | Find semantically matched forms | Returns scored pairs |
| PH4.S4 | Create `ingest_dialect` MCP tool | Register + ingest new dialect | Accepts custom configs |
| PH4.S5 | Wire integration points | Claude ↔ MCP ↔ LISP world | End-to-end test succeeds |

**Output:** `PH4_TOOL_DEFINITIONS.json`, `PH4_INTEGRATION_TEST.log`

---

## PHASE 5: TESTING, DOCS & RELEASE

**Phase ID:** `PH5`  
**Duration:** ~2-3 hours  
**Objective:** 100% test coverage, production documentation, v1.0.0 release

### Steps

| ID | Step | Deliverable | Verification |
|---|---|---|---|
| PH5.S1 | Unit tests (bridge layer) | Bridge test coverage 100% | `npm test` passes |
| PH5.S2 | Integration tests (MCP ↔ world) | Integration test coverage 90%+ | All scenarios pass |
| PH5.S3 | Performance benchmarks | Response time SLAs met | Bench report generated |
| PH5.S4 | Write production README | Multi-dialect setup guide | Clear 5-min quickstart |
| PH5.S5 | Tag v1.0.0 release | Git tag + GitHub release | Release notes posted |

**Output:** `PH5_TEST_REPORT.json`, `PH5_BENCHMARK_RESULTS.csv`, `v1.0.0` tag on GitHub

---

## AUTO-ADDRESSING SYSTEM

Each step can be referenced as:

```
PH{1-5}.S{1-5}
```

Example references:
- `PH1.S1` — Discover lisp-machine
- `PH2.S3` — Generate embeddings
- `PH3.S2` — Create equivalence classes
- `PH4.S5` — Wire integration points
- `PH5.S1` — Unit tests

All artifacts follow naming:
```
PH{phase}_DESCRIPTOR.{ext}
```

Example:
- `PH1_MANIFEST.json`
- `PH2_EMBEDDING_STATS.json`
- `PH3_SEMANTIC_MATCHES.json`
- `PH4_TOOL_DEFINITIONS.json`
- `PH5_TEST_REPORT.json`

---

## READY FOR XML PROMPT

Awaiting Jessica's XML specification that defines:

1. **Dialect specs** — McCarthy-1958, AppleSoft, custom
2. **Ingestion rules** — How to parse each dialect
3. **Semantic mapping** — Equivalence rules
4. **Integration configs** — MCP tool specs
5. **Success criteria** — Pass/fail per step

**Format expected:**

```xml
<build-spec version="1.0">
  <dialect name="McCarthy-1958">...</dialect>
  <dialect name="AppleSoft">...</dialect>
  <ingestion>...</ingestion>
  <semantic-mapping>...</semantic-mapping>
  <integration>...</integration>
</build-spec>
```

---

## EXECUTION FLOW

```
XML PROMPT (Jessica)
    ↓
SONNET (Claude Opus)
    ↓
PARSE & EXPAND per phase
    ↓
PH1 → PH2 → PH3 → PH4 → PH5
    ↓
v1.0.0 RELEASE
```

---

**Standing by for XML prompt. Ready to feed to Sonnet.**
