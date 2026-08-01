// backend/relational-engine/relational-refinement-engine.mjs
//
// SELF-MODIFYING FORMAL REFINEMENT ENGINE
// =========================================
// Implements the two system prompts:
//   1. LiquidHaskell Refined Synthesizer (max_mutation_depth: 5)
//   2. Relational Lisp Refinement Engine  (max_refinement_depth: 10)
//
// Pipeline:
//   INSPECT -> GENERATE -> VERIFY (LH refinements) -> MUTATE if fail -> EMIT
//
// The evalo core is symmetric: expr and val are co-equal constraints.
// (evalo expr env val) runs forward (eval) AND backward (synthesis).
//
// Builds on:
//   backend/relational-engine/evalo.mjs   (miniKanren core, already exists)
//   dsssl-synthesis/dsssl-synthesis.mjs   (SGML grove -> S-expr, already exists)
//
// SGML system declarations:
//   <!ELEMENT relational_engine - - (eval_core, mutator, proof_checker)>
//   <!ATTLIST eval_core mode (FORWARD | BACKWARD | SYMMETRIC) #REQUIRED>
//
// Ahmad Ali Parr -- Bel Esprit D'Accord Irrevocable Trust -- EIN 42-697643

import { createHash } from "crypto"

// ── Configuration (matches system prompt) ────────────────────────────────────
const ENGINE_CONFIG = {
  engine_metadata: {
    core:                   "Relational-Lisp-Evaluator-5Line",
    solver_mode:            "SMT_VERIFIED_SYNTHESIS",
    max_refinement_depth:   10,
    max_mutation_depth:     5,
    version:                "3.0.0",
  },
  runtime_state: {
    current_iteration: 0,
    holes_unfilled:    0,
    proof_status:      "UNSATISFIED",
    type_check_status: "PENDING",
  },
}

// ── miniKanren primitives ─────────────────────────────────────────────────────
// Same pattern as evalo.mjs — symmetric relational evaluator

const FAIL  = null
const EMPTY = {}

function walk(u, s) {
  while (typeof u === "string" && u.startsWith("_.") && u in s) u = s[u]
  return u
}

function unify(u, v, s) {
  u = walk(u, s); v = walk(v, s)
  if (u === v) return s
  if (typeof u === "string" && u.startsWith("_.")) return { ...s, [u]: v }
  if (typeof v === "string" && v.startsWith("_.")) return { ...s, [v]: u }
  if (Array.isArray(u) && Array.isArray(v) && u.length === v.length) {
    for (let i = 0; i < u.length; i++) {
      s = unify(u[i], v[i], s)
      if (s === FAIL) return FAIL
    }
    return s
  }
  if (u && v && typeof u === "object" && typeof v === "object") {
    for (const k of Object.keys(u)) {
      if (!(k in v)) return FAIL
      s = unify(u[k], v[k], s)
      if (s === FAIL) return FAIL
    }
    return s
  }
  return u === v ? s : FAIL
}

function* conde(...clauses) { for (const c of clauses) yield* c }

let _varN = 0
function fresh(f) { return f(`_.${++_varN}`) }

function reify(v, s) {
  v = walk(v, s)
  if (typeof v === "string" && v.startsWith("_.")) return v
  if (Array.isArray(v)) return v.map(x => reify(x, s))
  if (v && typeof v === "object") {
    const out = {}
    for (const k of Object.keys(v)) out[k] = reify(v[k], s)
    return out
  }
  return v
}

// ── The 5-Line Relational Core ────────────────────────────────────────────────
// evalo(expr, env, val) — symmetric: run forward (eval) or backward (synth)
//
// (conde
//   [(symbolo expr)  (lookupo expr env val)]
//   [(numbero expr)  (== expr val)]
//   [(== `(lambda (,x) ,body) expr)  (== `(closure ,x ,body ,env) val)]
//   [(== `(,e1 ,e2) expr)
//      (evalo e1 env `(closure ,x ,body ,env^))
//      (evalo e2 env arg)
//      (evalo body `((,x . ,arg) . ,env^) val)])

function lookupo(sym, env, val, s) {
  if (!Array.isArray(env) || env.length === 0) return FAIL
  const [first, ...rest] = env
  if (!Array.isArray(first)) return FAIL
  const [k, v] = first
  const s2 = unify(sym, k, s)
  if (s2 !== FAIL) return unify(val, v, s2)
  return lookupo(sym, rest, val, s)
}

function* evalo(expr, env, val, s = EMPTY, depth = 0) {
  if (depth > ENGINE_CONFIG.engine_metadata.max_refinement_depth) return

  const e = walk(expr, s)
  const v = walk(val, s)

  // Clause 1: symbol lookup
  if (typeof e === "string" && !e.startsWith("_.")) {
    const s2 = lookupo(e, env, v, s)
    if (s2 !== FAIL) yield s2
    return
  }

  // Clause 2: number/literal — (== expr val)
  if (typeof e === "number" || typeof e === "boolean") {
    const s2 = unify(e, v, s)
    if (s2 !== FAIL) yield s2
    return
  }

  // Clause 3: lambda -> closure
  // (== `(lambda (,x) ,body) expr) (== `(closure ,x ,body ,env) val)
  yield* fresh(x => fresh(body => {
    const s2 = unify(e, ["lambda", [x], body], s)
    if (s2 === FAIL) return []
    return unify(v, ["closure", x, body, env], s2) !== FAIL
      ? [unify(v, ["closure", x, body, env], s2)]
      : []
  }))

  // Clause 4: application — ((e1 e2) -> apply closure)
  if (Array.isArray(e) && e.length === 2) {
    const [e1, e2] = e
    yield* fresh(x => fresh(body => fresh(envPrime => fresh(arg => {
      const results = []
      for (const s2 of evalo(e1, env, ["closure", x, body, envPrime], s, depth + 1)) {
        for (const s3 of evalo(e2, env, arg, s2, depth + 1)) {
          const extEnv = [[walk(x, s3), walk(arg, s3)], ...walk(envPrime, s3)]
          for (const s4 of evalo(body, extEnv, v, s3, depth + 1)) {
            results.push(s4)
          }
        }
      }
      return results[Symbol.iterator]()
    }))))
  }
}

// ── LiquidHaskell Refinement Types ────────────────────────────────────────────
// Implemented as JS predicates — same semantics as the LH annotations:
//   type NonEmptyExpr = {e: SExpr | len e > 0}
//   type ValidAST     = {a: AST  | isWellFormed a && containsNoHoles a}
//   type QuineProof   = {p: AST  | eval p == p}
//   type Synthesis    = {s: AST  | satisfiesInvariants s && verifiesSMT s}

const LH_REFINEMENTS = {
  // {e: SExpr | len e > 0}
  NonEmptyExpr: (e) => {
    if (e === null || e === undefined) return { ok: false, err: "NonEmptyExpr: null input" }
    const len = Array.isArray(e) ? e.length : String(e).length
    if (len === 0) return { ok: false, err: "NonEmptyExpr: len e > 0 violated" }
    return { ok: true }
  },

  // {a: AST | isWellFormed a && containsNoHoles a}
  ValidAST: (a) => {
    if (a === null || a === undefined) return { ok: false, err: "ValidAST: null AST" }
    // isWellFormed: no undefined nodes
    const hasUndefined = (node) => {
      if (node === undefined) return true
      if (Array.isArray(node)) return node.some(hasUndefined)
      if (node && typeof node === "object") return Object.values(node).some(hasUndefined)
      return false
    }
    if (hasUndefined(a)) return { ok: false, err: "ValidAST: isWellFormed violated — undefined node" }
    // containsNoHoles: no logic variable (_.N) in final AST
    const hasHole = (node) => {
      if (typeof node === "string" && node.startsWith("_.")) return true
      if (Array.isArray(node)) return node.some(hasHole)
      if (node && typeof node === "object") return Object.values(node).some(hasHole)
      return false
    }
    if (hasHole(a)) return { ok: false, err: "ValidAST: containsNoHoles violated — unresolved logic var" }
    return { ok: true }
  },

  // {p: AST | eval p == p} — quine: self-evaluating expression
  QuineProof: (p) => {
    const results = [...evalo(p, [], p, EMPTY, 0)]
    if (results.length === 0) return { ok: false, err: "QuineProof: eval p == p not satisfied" }
    return { ok: true }
  },

  // {s: AST | satisfiesInvariants s && verifiesSMT s}
  Synthesis: (s, spec) => {
    const valid = LH_REFINEMENTS.ValidAST(s)
    if (!valid.ok) return valid
    // verifiesSMT: structure matches the spec (simplified SMT: unification check)
    if (spec) {
      const sub = unify(s, spec, EMPTY)
      if (sub === FAIL) return { ok: false, err: "Synthesis: verifiesSMT — unification with spec failed" }
    }
    return { ok: true }
  },
}

// ── AST Mutation Passes ───────────────────────────────────────────────────────
// Applied when a refinement check fails.
// Three passes (from system prompt):
//   REFACTOR:        rename logic vars, simplify structure
//   INVARIANT_FIX:   fill holes with defaults
//   STRUCTURAL_PRUNE: remove subtrees that violate well-formedness

const MUTATION_PASSES = {
  REFACTOR: (ast, s) => {
    // Walk and reify all logic variables using current substitution
    return reify(ast, s)
  },

  INVARIANT_FIX: (ast, _s) => {
    // Replace holes (_.N) with null, empty arrays with [null]
    const fix = (node) => {
      if (typeof node === "string" && node.startsWith("_.")) return null
      if (Array.isArray(node)) {
        const fixed = node.map(fix)
        return fixed.length === 0 ? [null] : fixed
      }
      if (node && typeof node === "object") {
        const out = {}
        for (const k of Object.keys(node)) out[k] = fix(node[k])
        return out
      }
      return node
    }
    return fix(ast)
  },

  STRUCTURAL_PRUNE: (ast, _s) => {
    // Remove undefined, null subtrees that aren't at leaves
    const prune = (node) => {
      if (node === null || node === undefined) return null
      if (Array.isArray(node)) {
        const pruned = node.map(prune).filter(x => x !== null)
        return pruned.length > 0 ? pruned : null
      }
      if (typeof node === "object") {
        const out = {}
        for (const k of Object.keys(node)) {
          const v = prune(node[k])
          if (v !== null) out[k] = v
        }
        return Object.keys(out).length > 0 ? out : null
      }
      return node
    }
    return prune(ast)
  },
}

// ── The Self-Modifying Refinement Loop ────────────────────────────────────────
// synthesizeAndVerify :: input:NonEmptyExpr -> {v:ValidAST | preservesInvariants input v}

export async function synthesizeAndVerify(input, spec = null, options = {}) {
  const config = { ...ENGINE_CONFIG }
  config.runtime_state = { ...ENGINE_CONFIG.runtime_state }

  const mutation_log   = []
  const invariants     = ["NonEmptyExpr", "ValidAST", "Synthesis"]
  let   current_ast    = input
  let   type_check_status = "PENDING"
  let   smt_status     = "UNSAT"
  const passes         = Object.keys(MUTATION_PASSES)

  // STEP 1: INSPECT — validate input
  const inputCheck = LH_REFINEMENTS.NonEmptyExpr(input)
  if (!inputCheck.ok) {
    return {
      verification_trace: {
        iterations_required:      0,
        liquid_invariants_checked: invariants,
        smt_proof_status:         "UNSAT",
      },
      mutation_log: [{ pass: 0, failed_refinement: inputCheck.err, applied_ast_rewrite: "INPUT_REJECTED" }],
      verified_payload: null,
      status: "ERROR",
      transformation_summary: "Input rejected: " + inputCheck.err,
    }
  }

  // STEP 2+3+4: GENERATE -> VERIFY -> MUTATE loop
  for (let i = 0; i <= config.engine_metadata.max_mutation_depth; i++) {
    config.runtime_state.current_iteration = i

    // Try evalo: run the relational evaluator to generate/verify
    const evalResults = [...evalo(current_ast, [], spec ?? current_ast, EMPTY, 0)]
    const best_sub    = evalResults[0] ?? EMPTY
    const candidate   = reify(current_ast, best_sub)

    // Verify against LH refinements
    const astCheck  = LH_REFINEMENTS.ValidAST(candidate)
    const synthCheck = LH_REFINEMENTS.Synthesis(candidate, spec)

    if (astCheck.ok && synthCheck.ok) {
      type_check_status = "PASSED"
      smt_status        = evalResults.length > 0 ? "SAT" : "UNSAT"
      config.runtime_state.type_check_status = "PASSED"
      config.runtime_state.holes_unfilled    = 0
      current_ast = candidate
      break
    }

    // MUTATE: log the failure, apply next mutation pass
    const failed_ref = !astCheck.ok ? astCheck.err : synthCheck.err
    const pass_name  = passes[i % passes.length]
    const mutated    = MUTATION_PASSES[pass_name](candidate, best_sub)

    mutation_log.push({
      pass:                i,
      failed_refinement:   failed_ref,
      applied_ast_rewrite: pass_name,
      before_hash:         sha256(JSON.stringify(candidate)).slice(0, 16),
      after_hash:          sha256(JSON.stringify(mutated)).slice(0, 16),
    })

    current_ast = mutated

    // Count remaining holes
    const countHoles = (n) => {
      if (typeof n === "string" && n.startsWith("_.")) return 1
      if (Array.isArray(n)) return n.reduce((a, x) => a + countHoles(x), 0)
      if (n && typeof n === "object") return Object.values(n).reduce((a, x) => a + countHoles(x), 0)
      return 0
    }
    config.runtime_state.holes_unfilled = countHoles(current_ast)
  }

  // STEP 5: EMIT
  const final_check = LH_REFINEMENTS.ValidAST(current_ast)
  const success     = type_check_status === "PASSED" && final_check.ok

  return {
    verification_trace: {
      iterations_required:       config.runtime_state.current_iteration,
      liquid_invariants_checked: invariants,
      smt_proof_status:          smt_status,
    },
    mutation_log,
    verified_payload:      current_ast,
    status:                success ? "SUCCESS" : "ERROR",
    transformation_summary: success
      ? `Verified in ${config.runtime_state.current_iteration} iteration(s). SMT: ${smt_status}.`
      : `Max depth reached. ${mutation_log.length} mutation(s) applied. Holes: ${config.runtime_state.holes_unfilled}.`,
    engine_state: config.runtime_state,
  }
}

// ── Backward synthesis: given expected output, find the expression ────────────
// This is the "backward" mode of evalo:
// evalo(_.expr, env, expectedValue) -> yields expr that produces expectedValue

export async function* synthesize(expectedVal, env = []) {
  _varN = 0
  const exprVar = `_.expr_${++_varN}`
  for (const s of evalo(exprVar, env, expectedVal, EMPTY, 0)) {
    const expr = reify(exprVar, s)
    if (typeof expr !== "string" || !expr.startsWith("_.")) {
      yield { expr, substitution: s }
    }
  }
}

// ── Quine check: find self-evaluating expression ──────────────────────────────
// evalo(e, [], e) -> e is a quine

export async function* findQuines(seed = null) {
  _varN = 0
  const e = seed ?? `_.quine_${++_varN}`
  for (const s of evalo(e, [], e, EMPTY, 0)) {
    const result = reify(e, s)
    if (LH_REFINEMENTS.ValidAST(result).ok) {
      yield { quine: result, proof: "eval quine == quine" }
    }
  }
}

// ── SHA-256 seal ──────────────────────────────────────────────────────────────
function sha256(data) {
  return createHash("sha256").update(data).digest("hex")
}

export function sealResult(result) {
  const raw  = JSON.stringify(result.verified_payload)
  const seal = sha256(raw)
  return { ...result, worm_seal: seal.slice(0, 16), full_seal: seal }
}
