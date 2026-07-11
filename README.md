# SNAPKITTY CLOJURE LISP BRIDGE

> **A ClojureScript MCP knowledge-base server (Solarium) — extracted, audited, and quarantined.**
>
> `Ω ← TRUST ∧ CODE` — and this artifact failed the TRUST half.

Part of the **SNAPKITTYWEST** sovereign-compute constellation, retained as the negative-space
witness: the record of what the WORM chain *rejects*.

> **STATUS: ARCHIVED — CLASSIFIED PUMP AND DUMP — DO NOT USE IN PRODUCTION**
> Extracted from `SNAPKITTYWEST/solarium`. Audit score **-9.8/10**. Zero test coverage.
> 5+ critical vulnerabilities (CVSS up to 9.8). See [`audit/AUDIT-REPORT.md`](audit/AUDIT-REPORT.md).

---

## OVERVIEW

This repository is the **Solarium** agent implementation: a ClojureScript
(shadow-cljs → Node) **MCP server** that provides a shared, semantically-searchable knowledge
base backed by the **Qdrant** vector database and **local ONNX embeddings**, plus a
server-rendered HTML **dashboard**. It corresponds to the *Solarium — Semantic Knowledge* agent
role in the MATHLIB5 constellation.

It is preserved here for **historical reference, audit trail, and learning** only. Every source
line was audited line-by-line and found defective, unsafe, or incomplete. It is not sealed to
the WORM chain and must not be trusted.

## WHAT IT IS

- A **shadow-cljs** project (`shadow-cljs.edn`, `deps.edn`) producing two Node scripts:
  - `:server` → `out/server.js` (MCP server, entry `doomsun.solarium.main/main!`)
  - `:dashboard` → `out/dashboard.js` (HTTP dashboard, entry `doomsun.solarium.dashboard.main/main!`)
- Dependencies (`package.json`): `@modelcontextprotocol/sdk`, `@huggingface/transformers`,
  `onnxruntime-node`, `marked`, `zod`; Tailwind CSS v4 for the dashboard.
- A Tailwind-based dark **"Doomsun" dashboard theme** (`src/css/dashboard.css`, 753 lines)
  with sidebar, stat cards, search, tag cloud, prose/markdown, and chart styling.
- Cross-language **architecture meta-specs** in Janet and Hy (both marking every component
  `UNVERIFIED`).

## ARCHITECTURE / COMPONENTS

The MCP server exposes document, search, and tag tools; ingestion chunks text, embeds it via a
local ONNX model, and upserts vectors into Qdrant. The dashboard renders overview/search/
documents/analytics views over the same store.

```
snapkitty-clojure-lisp-bridge/
├── src/doomsun/solarium/
│   ├── main.cljs                  # MCP server entry
│   ├── server.cljs                # Tool registration + wrap-handler
│   ├── config.cljs                # Env-based configuration
│   ├── util.cljs                  # Utilities (logging, ids, tool-result)
│   ├── chunking.cljs              # Text chunking (chunk-max / overlap)
│   ├── tools/
│   │   ├── documents.cljs         # store/read/update/delete document tools
│   │   ├── search.cljs            # semantic search tool
│   │   └── tags.cljs              # tag management tools
│   ├── qdrant/
│   │   ├── client.cljs            # Qdrant HTTP client
│   │   ├── collections.cljs       # collection management
│   │   └── points.cljs            # vector point upsert/query
│   ├── embedding/
│   │   ├── model.cljs             # ONNX model load/download
│   │   └── pipeline.cljs          # embedding pipeline
│   └── dashboard/
│       ├── main.cljs server.cljs http.cljs routes.cljs
│       ├── sse.cljs static.cljs html.cljs
│       └── views/                 # layout, overview, search, documents, analytics, random
├── src/css/dashboard.css          # Tailwind v4 Doomsun theme
├── spec/architecture.janet        # meta-spec (UNVERIFIED)
├── spec/architecture.hy           # meta-spec (UNVERIFIED)
├── audit/AUDIT-REPORT.md          # 664-line line-by-line audit
├── legacy/WARNING.md              # do-not-use notice
├── .github/repo-config.yml        # archived=true, issues/PRs disabled
├── deps.edn  package.json  shadow-cljs.edn
```

### Component map

| Path | Purpose |
|------|---------|
| `doomsun/solarium/server.cljs` | Registers MCP tools; `wrap-handler` (⚠ no input validation — VULN-2026-001) |
| `doomsun/solarium/config.cljs` | Loads config from env (⚠ empty `qdrant-key` default — VULN-2026-002) |
| `doomsun/solarium/qdrant/client.cljs` | Qdrant requests (⚠ API key optional — VULN-2026-003) |
| `doomsun/solarium/embedding/model.cljs` | ONNX model load + auto-download (⚠ no checksum — VULN-2026-004) |
| `doomsun/solarium/tools/documents.cljs` | Document ingestion (⚠ no rate limiting — VULN-2026-005) |
| `doomsun/solarium/dashboard/html.cljs` | HTML rendering (⚠ `dangerouslySetInnerHTML` XSS) |
| `spec/architecture.*` | Janet/Hy meta-specs; all layers/agents flagged UNVERIFIED |

## HOW IT FITS THE CONSTELLATION

- **Plasma Gate / Ed25519** — the meta-spec references the constellation trust primitives
  (`Plasma Gate (Ed25519)`, `AES-256-GCM`, `SHA-256 Merkle`) but marks each **UNAUDITED /
  MISCONFIGURED / BROKEN**. This repo is *outside* the gate.
- **WORM chain / Bifrost** — this artifact is **not WORM-sealed**. It exists as the audit trail
  proving the chain refuses to seal unverified, untested code.
- **P/NP swarm** — the audit itself is the P-time verifier verdict: a witness (the code dump)
  was submitted and **failed verification**, so `universeSum` does not advance.
- **3-witness verification** — the artifact carries zero passing witnesses: the two test files
  (`chunking_test.cljs`, `html_test.cljs`) are stubs with no assertions. Under the constellation's
  multi-witness rule, it cannot converge.

## BUILD / USAGE / INSTALL

> **Do not deploy.** These commands document how the artifact was built; use for inspection only.

```bash
npm install
npm run build           # shadow-cljs release server → out/server.js
npm run build:dashboard # tailwind css + shadow-cljs release dashboard
npm run build:all       # both server and dashboard
npm run watch           # dev watch of the server build
npm test                # shadow-cljs compile test → node out/tests.js (stubs only)
```

Configuration is read from environment variables (see `config.cljs`): `QDRANT_URL`,
`QDRANT_API_KEY`, `COLLECTION_NAME`, `MODEL_NAME`, `CHUNK_MAX_CHARS`, `CHUNK_OVERLAP`.

### If you must reuse the concept

Rewrite from scratch. Add input validation (Zod is present but unused), require an authenticated
Qdrant key, verify model downloads by SHA-256, rate-limit ingestion, escape all HTML, and reach
100% test coverage before anything approaches the Plasma Gate.

## KEY FILES REFERENCE

| File | Why it matters |
|------|----------------|
| `audit/AUDIT-REPORT.md` | Full 664-line audit: vulnerabilities, findings, file-by-file scores |
| `legacy/WARNING.md` | Canonical do-not-use notice |
| `.github/repo-config.yml` | Enforces archived state (issues/PRs/wiki/discussions off) |
| `shadow-cljs.edn` | Build targets for server, dashboard, and test |
| `src/css/dashboard.css` | The Doomsun dashboard theme |
| `spec/architecture.janet` / `spec/architecture.hy` | UNVERIFIED meta-specs + contributor record |

## LICENSE

Historical/legacy artifact of SNAPKITTYWEST. Read-only and archived; all issues, pull requests,
and discussions are disabled.
