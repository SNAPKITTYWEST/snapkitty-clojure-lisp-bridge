// backend/ahmad-docking/machine-client.mjs
//
// Ahmad Docking Machine Client
// High-level API for metatron.mjs and bob-bridge to use the Lisp machine.
//
// Usage:
//   import { evaluate, handshake, seal } from './machine-client.mjs'
//
// Ahmad Ali Parr -- Bel Esprit D'Accord Irrevocable Trust -- EIN 42-697643

import { evalLisp, evalSexp, worldDump, parseLisp, ahmadDock } from './lisp-bridge.mjs'
import { createHash } from 'crypto'

const PHI = (1 + Math.sqrt(5)) / 2

// ── evaluate ─────────────────────────────────────────────────────────────────
// Evaluate a Lisp source string. Returns result + WORM seal.
export function evaluate (src) {
  return evalLisp(src)
}

// ── handshake ─────────────────────────────────────────────────────────────────
// Emit a BOB-protocol S-expression handshake entry from a Lisp evaluation.
// Matches the lisp-handshake.json protocol format.
export function handshake (src, hat = 'lisp') {
  const { result, tick, seal } = evalLisp(src)
  return {
    agent:  'METATRON-LISP',
    hat,
    ts:     Date.now(),
    tick,
    src,
    result: JSON.stringify(result),
    seal,
    phi_weight: PHI ** 5,
  }
}

// ── seal ─────────────────────────────────────────────────────────────────────
// WORM-seal an arbitrary value through the Lisp machine.
export function seal (value) {
  return ahmadDock(JSON.stringify(value))
}

// ── batchEval ────────────────────────────────────────────────────────────────
// Evaluate multiple expressions and return results + chain seal.
export function batchEval (sources) {
  const results = sources.map(src => evalLisp(src))
  const chain   = createHash('sha256')
    .update(results.map(r => r.seal).join(':'))
    .digest('hex')
    .slice(0, 16)
  return { results, chain_seal: chain }
}

// ── snapshot ─────────────────────────────────────────────────────────────────
// Take a world dump snapshot and return it.
export function snapshot () {
  return worldDump()
}

// ── runSexpHandshake ──────────────────────────────────────────────────────────
// Given a list of S-expression strings, evaluate all and produce a
// bob-handshake compatible sexp output.
export function runSexpHandshake (sexpSources) {
  const entries = sexpSources.map((src, i) => {
    const { result, tick, seal } = evalLisp(src)
    return `(entry (index ${i}) (src "${src.replace(/"/g, "'")}") (result ${JSON.stringify(result)}) (tick ${tick}) (seal "${seal}"))`
  })
  const dump = worldDump()
  return [
    `(machine-handshake`,
    `  (agent "${dump.agent}")`,
    `  (tick ${dump.tick})`,
    `  (world-seal "${dump.seal}")`,
    ...entries.map(e => `  ${e}`),
    `)`
  ].join('\n')
}
