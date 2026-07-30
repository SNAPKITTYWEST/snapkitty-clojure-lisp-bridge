# Tech Debt Removal — Snapkitty Clojure LISP Bridge

**Status:** CLEAN SLATE BUILD  
**Date:** 2026-07-25  
**Target:** Unified LISP-Clojure world combining all SnapKitty LISP implementations

---

## Directories to Delete

```
audit/                    # Broken audit trail — replaced by new verification
legacy/                   # Do-not-use notices — entire basis invalid now
spec/architecture.*       # UNVERIFIED meta-specs — replaced by clean architecture
```

**Reason:** These are witnesses of failed verification from the previous fork. They block rebuilding from first principles.

---

## Vulnerabilities to Fix (Immediate)

| VULN | File | Issue | Fix |
|------|------|-------|-----|
| VULN-2026-001 | `server.cljs` | No input validation on MCP wrap-handler | Add Zod schema validation on all tool inputs |
| VULN-2026-002 | `config.cljs` | Empty `qdrant-key` default | Require QDRANT_API_KEY env var; fail fast if missing |
| VULN-2026-003 | `qdrant/client.cljs` | API key optional in requests | Enforce auth header on all Qdrant calls |
| VULN-2026-004 | `embedding/model.cljs` | No checksum on model download | Verify SHA-256 of ONNX model before use |
| VULN-2026-005 | `tools/documents.cljs` | No rate limiting on ingestion | Add token bucket rate limiter (10 docs/sec) |
| VULN-2026-006 | `dashboard/html.cljs` | XSS via `dangerouslySetInnerHTML` | Use Hiccup safe rendering; escape all user content |

---

## New Clean Architecture

### Layer 1: MCP Core
- `src/snapkitty/lisp/mcp/server.cljs` — MCP server (verified input)
- `src/snapkitty/lisp/mcp/tools.cljs` — Tool definitions (Zod-validated)
- `src/snapkitty/lisp/mcp/transport.cljs` — Stdio transport wrapper

### Layer 2: Knowledge Store
- `src/snapkitty/lisp/knowledge/qdrant.cljs` — Qdrant client (auth required)
- `src/snapkitty/lisp/knowledge/embedding.cljs` — ONNX embeddings (verified)
- `src/snapkitty/lisp/knowledge/store.cljs` — Document storage + versioning

### Layer 3: LISP Bridge
- `src/snapkitty/lisp/bridge/reader.cljs` — Read LISP code (Clojure reader)
- `src/snapkitty/lisp/bridge/compiler.cljs` — LISP → knowledge graph
- `src/snapkitty/lisp/bridge/macros.cljs` — LISP macro expansion

### Layer 4: Integration Points
- `src/snapkitty/lisp/integration/lisp-machine.cljs` — Link to `lisp-machine` repo
- `src/snapkitty/lisp/integration/apple-ii.cljs` — Link to `apple-ii-universal-machine`
- `src/snapkitty/lisp/integration/world.cljs` — Unified LISP-Clojure world

### Testing
- `test/snapkitty/lisp/mcp_test.cljs` — MCP integration
- `test/snapkitty/lisp/knowledge_test.cljs` — Store + embedding
- `test/snapkitty/lisp/bridge_test.cljs` — LISP reader + compiler
- `test/snapkitty/lisp/integration_test.cljs` — End-to-end

---

## Build Configuration (Clean)

**deps.edn:**
```edn
{:paths ["src"]
 :deps {org.clojure/clojure {:mvn/version "1.11.1"}
        org.clojure/clojurescript {:mvn/version "1.10.914"}
        thheller/shadow-cljs {:mvn/version "2.28.13"}
        com.taoensso/promesa {:mvn/version "11.0.676"}
        zod {:npm/package "@zod/core" :npm/version "0.2.0"}
        onnxruntime {:npm/package "onnxruntime-node" :npm/version "1.18.1"}}}
```

**shadow-cljs.edn:**
```edn
{:source-paths ["src"]
 :builds
 {:server {:target :node-script
           :output-to "out/server.js"
           :main snapkitty.lisp.mcp/start!}
  :tests {:target :node-test
          :output-to "out/tests.js"
          :test-dir "test"}}}
```

**package.json:**
```json
{
  "name": "snapkitty-clojure-lisp-bridge",
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "build": "shadow-cljs release server",
    "test": "shadow-cljs compile tests && node out/tests.js",
    "watch": "shadow-cljs watch server tests",
    "clean": "rm -rf out .shadow-cljs"
  },
  "dependencies": {
    "@modelcontextprotocol/sdk": "^1.0.0",
    "onnxruntime-node": "^1.18.1",
    "zod": "^3.22.4"
  }
}
```

---

## Deletion Plan (Safe)

1. **Backup current state:**
   ```bash
   git tag archive/pre-cleanup
   ```

2. **Delete tech debt directories:**
   ```bash
   git rm -r audit/ legacy/ spec/
   ```

3. **Rewrite README** (clean new vision)

4. **Restructure source tree** (rename from `doomsun/solarium` → `snapkitty/lisp`)

5. **Fix all 6 vulnerabilities** (new implementations)

6. **Create test scaffold** (100% coverage target)

7. **Commit as v1.0.0-clean:**
   ```bash
   git commit -m "BREAKING: Clean slate rebuild — remove all tech debt, fix vulnerabilities, establish LISP bridge"
   ```

---

## Integration Checklist

- [ ] Remove `audit/`, `legacy/`, `spec/`
- [ ] Rename namespace from `doomsun.solarium` → `snapkitty.lisp`
- [ ] Rewrite `config.cljs` — enforce required env vars
- [ ] Rewrite `server.cljs` — add Zod input validation
- [ ] Rewrite `qdrant/client.cljs` — enforce auth header
- [ ] Rewrite `embedding/model.cljs` — add SHA-256 verification
- [ ] Rewrite `tools/documents.cljs` — add rate limiting
- [ ] Rewrite `dashboard/html.cljs` — escape all HTML
- [ ] Create `bridge/reader.cljs` — LISP code reader
- [ ] Create `bridge/compiler.cljs` — LISP → knowledge graph
- [ ] Create test suite (100% core coverage)
- [ ] Write new README (clean vision + integration plan)
- [ ] Tag and commit v1.0.0-clean
- [ ] Begin LISP world unification work

---

**Ready to proceed with deletion?**
