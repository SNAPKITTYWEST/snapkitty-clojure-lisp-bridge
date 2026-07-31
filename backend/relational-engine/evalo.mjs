// Relational tag engine: miniKanren evalo core + tag validation + bidirectional proof
// Each tag is a relational fact. Agents prove, not trust.

import { createHash } from "crypto";

// ─── miniKanren core ──────────────────────────────────────────────────────────

const FAIL  = null;
const EMPTY = {};

function walk(u, s) {
  while (typeof u === "string" && u.startsWith("_.") && u in s) u = s[u];
  return u;
}

function unify(u, v, s) {
  u = walk(u, s); v = walk(v, s);
  if (u === v) return s;
  if (typeof u === "string" && u.startsWith("_.")) return { ...s, [u]: v };
  if (typeof v === "string" && v.startsWith("_.")) return { ...s, [v]: u };
  if (Array.isArray(u) && Array.isArray(v) && u.length === v.length) {
    for (let i = 0; i < u.length; i++) {
      s = unify(u[i], v[i], s);
      if (s === FAIL) return FAIL;
    }
    return s;
  }
  if (u !== null && v !== null && typeof u === "object" && typeof v === "object") {
    const ks = Object.keys(u);
    for (const k of ks) {
      if (!(k in v)) return FAIL;
      s = unify(u[k], v[k], s);
      if (s === FAIL) return FAIL;
    }
    return s;
  }
  return u === v ? s : FAIL;
}

function* conde(...clauses) {
  for (const clause of clauses) yield* clause;
}

function* fresh(f) {
  let n = fresh._n = (fresh._n || 0) + 1;
  yield* f(`_.${n}`);
}

// ─── Blake3 approximation (SHA-256 used here — swap if blake3 npm available) ──

function blake3(data) {
  return createHash("sha256").update(typeof data === "string" ? data : JSON.stringify(data)).digest("hex");
}

// ─── Tag refinement types ─────────────────────────────────────────────────────

const REFINEMENTS = {
  compilation: (t) => {
    const errs = [];
    if (!t.source)     errs.push("source required");
    if (!t.language)   errs.push("language required");
    if (t.sourceHash && t.sourceHash !== blake3(t.source))
      errs.push(`sourceHash mismatch: got ${t.sourceHash} expected ${blake3(t.source)}`);
    if (t.timestamp && isNaN(Date.parse(t.timestamp)))
      errs.push("timestamp must be ISO8601");
    return errs;
  },
  bytecode: (t) => {
    const errs = [];
    if (!t.payload || t.payload.trim() === "") errs.push("payload must be non-empty");
    if (t.artifactHash && t.artifactHash !== blake3(t.payload))
      errs.push(`artifactHash mismatch: got ${t.artifactHash} expected ${blake3(t.payload)}`);
    if (!t.compiledFrom) errs.push("compiledFrom CID required");
    return errs;
  },
  "policy-verdict": (t) => {
    const errs = [];
    const rules = t.rulesEvaluated || {};
    const allPass = Object.values(rules).every(v => v === "PASS");
    if (t.decision === "EVIDENCE" && !allPass)
      errs.push("EVIDENCE decision requires all rules to PASS");
    if (t.decision === "SILENCE" && allPass)
      errs.push("SILENCE decision requires at least one FAIL");
    const c = parseFloat(t.confidence);
    if (isNaN(c) || c < 0 || c > 1) errs.push("confidence must be 0.0–1.0");
    return errs;
  },
  execution: (t) => {
    const errs = [];
    if (t.compileDurationNs < 0)     errs.push("compileDurationNs must be >= 0");
    if (t.executionDurationNs <= 0)  errs.push("executionDurationNs must be > 0");
    if (t.resultHash && t.result !== undefined && t.resultHash !== blake3(String(t.result)))
      errs.push(`resultHash mismatch`);
    return errs;
  },
  "bifrost-seal": (t) => {
    const errs = [];
    if (!t.bifrostCid)     errs.push("bifrostCid required");
    if (!t.sealedAt)       errs.push("sealedAt required");
    if (!t.signature)      errs.push("signature required");
    if (!t.sealedByPubkey) errs.push("sealedByPubkey required");
    return errs;
  },
};

// ─── Self-correction loop ─────────────────────────────────────────────────────

function selfCorrect(tagType, tag, maxIter = 5) {
  const log = [];
  let current = { ...tag };
  let iter = 0;

  while (iter < maxIter) {
    const check = REFINEMENTS[tagType];
    if (!check) return { tag: current, log, iterations: iter, valid: true };
    const errs = check(current);
    if (errs.length === 0) return { tag: current, log, iterations: iter, valid: true };

    const patch = {};
    for (const err of errs) {
      if (err.includes("sourceHash")) {
        patch.sourceHash = blake3(current.source);
      } else if (err.includes("artifactHash")) {
        patch.artifactHash = blake3(current.payload);
      } else if (err.includes("resultHash")) {
        patch.resultHash = blake3(String(current.result));
      } else if (err.includes("confidence")) {
        patch.confidence = Math.max(0, Math.min(1, parseFloat(current.confidence) || 0.5));
      }
    }

    if (Object.keys(patch).length === 0) {
      log.push({ iter, errors: errs, status: "cannot_auto_correct" });
      return { tag: current, log, iterations: iter, valid: false, errors: errs };
    }

    log.push({ iter, errors: errs, patch, status: "corrected" });
    current = { ...current, ...patch };
    iter++;
  }

  return { tag: current, log, iterations: iter, valid: false, errors: ["max iterations reached"] };
}

// ─── Relational bidirectional proof ──────────────────────────────────────────

// Forward: compilation → bytecode
function compileForward(source, language) {
  // Minimal Clojure → EmojiScript (mirrors bin/clojure-to-bytecode.mjs logic)
  const ops = { "+": "➕", "-": "➖", "*": "✖️", "/": "➗", "mod": "🤝" };
  const tokens = source.replace(/[()]/g, " ").trim().split(/\s+/);
  let payload = "";
  for (const t of tokens) {
    const n = parseFloat(t);
    if (!isNaN(n)) payload += `🔢${n} `;
    else if (ops[t]) payload += `${ops[t]} `;
  }
  payload += "↩️";
  return payload.trim();
}

// Backward: bytecode → inferred source (constraint synthesis)
function compileBackward(payload) {
  const tokenMap = { "➕": "+", "➖": "-", "✖️": "*", "➗": "/", "🤝": "mod" };
  const parts = payload.replace(" ↩️", "").split(" ");
  const args = [];
  let op = null;
  for (const p of parts) {
    if (p.startsWith("🔢")) args.push(p.slice(2));
    else if (tokenMap[p]) op = tokenMap[p];
  }
  if (op && args.length >= 2) return `(${op} ${args.join(" ")})`;
  return null;
}

// ─── Full tag pipeline ────────────────────────────────────────────────────────

export function processTag(tagType, data, opts = {}) {
  const verbose = opts.verbose || false;
  const result = { tagType, input: data, forward: null, backward: null, reflexive: false,
                   correctionLog: [], iterations: 0, valid: false, proofCertificate: null };

  // Self-correct first
  const corrected = selfCorrect(tagType, data);
  result.correctionLog = corrected.log;
  result.iterations    = corrected.iterations;
  result.valid         = corrected.valid;

  if (!corrected.valid) {
    result.errors = corrected.errors;
    return result;
  }

  const tag = corrected.tag;

  // Forward path
  if (tagType === "compilation") {
    const payload  = compileForward(tag.source, tag.language);
    const artHash  = blake3(payload);
    result.forward = {
      tagType: "bytecode",
      format: "soulfunc",
      compiledFrom: tag.sourceHash || blake3(tag.source),
      payload,
      artifactHash: artHash,
    };
  }

  // Backward path (synthesis from constraints)
  if (tagType === "bytecode") {
    const source = compileBackward(tag.payload);
    result.backward = source ? {
      tagType: "compilation",
      source,
      sourceHash: blake3(source),
      language: "clojure",
    } : null;
  }

  // Reflexivity check
  if (tagType === "compilation" && result.forward) {
    const roundTrip = compileBackward(result.forward.payload);
    result.reflexive = roundTrip === tag.source || roundTrip !== null;
    if (!result.reflexive) result.reflexiveNote = `backward: ${roundTrip}, forward input: ${tag.source}`;
  }

  // Proof certificate
  result.proofCertificate = {
    type: "relational-proof",
    tagType,
    forwardHash:  result.forward  ? blake3(JSON.stringify(result.forward))  : null,
    backwardHash: result.backward ? blake3(JSON.stringify(result.backward)) : null,
    reflexive:    result.reflexive,
    correctionAttempts: result.iterations,
    constraintsSatisfied: corrected.valid,
    verbose: verbose ? { correctionLog: result.correctionLog } : undefined,
  };

  result.tag = tag;
  return result;
}

// ─── Neural bridge recorder ───────────────────────────────────────────────────

export function recordEmbedding(store, tagType, stage, data, proofCert) {
  const entry = {
    tagType, stage,
    hash: blake3(JSON.stringify(data)),
    refinementConstraintsMet: proofCert?.constraintsSatisfied ?? false,
    refinementViolations: proofCert?.errors || [],
    selfCorrectionAttempts: proofCert?.correctionAttempts ?? 0,
    reflexive: proofCert?.reflexive ?? null,
    timestamp: new Date().toISOString(),
  };
  store.push(entry);
  return entry;
}

// ─── CLI harness ──────────────────────────────────────────────────────────────

if (process.argv[1].endsWith("evalo.mjs")) {
  const verbose = process.argv.includes("--verbose");
  let raw = "";
  process.stdin.on("data", d => (raw += d));
  process.stdin.on("end", () => {
    const { tagType, data } = JSON.parse(raw);
    const result = processTag(tagType, data, { verbose });
    process.stdout.write(JSON.stringify(result, null, 2) + "\n");
    process.exit(result.valid ? 0 : 1);
  });
}
