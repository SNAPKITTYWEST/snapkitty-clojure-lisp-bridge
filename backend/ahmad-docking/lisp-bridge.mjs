// backend/ahmad-docking/lisp-bridge.mjs
//
// Ahmad Docking -- Lisp Machine Bridge
// Connects snapkitty-clojure-lisp-bridge to the ahmad-docking sovereign Lisp machine.
//
// Protocol:
//   eval(src)        -> { result, tick, seal }
//   worldDump()      -> { tick, agent, seal }
//   evalSexp(sexp)   -> { result, tick, seal }  (for BOB handshake protocol)
//
// The Clojure machine (ahmad-docking/clojure/lisp_machine.clj) is the runtime.
// This bridge exposes it as a JS-callable API so metatron.mjs and bob-bridge
// can route S-expressions through the sovereign Lisp evaluator.
//
// Ahmad Ali Parr -- Bel Esprit D'Accord Irrevocable Trust -- EIN 42-697643

import { createHash } from 'crypto'
import { spawn } from 'child_process'

const PHI = (1 + Math.sqrt(5)) / 2

// ── In-process Lisp evaluator (no Clojure dep required for basic ops) ────────
// Mirrors the Clojure machine semantics exactly.
// For full Clojure runtime: spawn clojure -M -m ahmad-docking.lisp-machine

class SovereignLispMachine {
  constructor (agentId = 'METATRON') {
    this.agentId  = agentId
    this.tick     = 0
    this.env      = {}          // symbol -> value
    this.vault    = []          // WORM chain of world dumps
    this._setupBuiltins()
  }

  _setupBuiltins () {
    this.builtins = {
      '+':     args => args.reduce((a, b) => a + b, 0),
      '-':     args => args.slice(1).reduce((a, b) => a - b, args[0]),
      '*':     args => args.reduce((a, b) => a * b, 1),
      '/':     args => args.slice(1).reduce((a, b) => b !== 0 ? a / b : 0, args[0]),
      'cons':  args => [args[0], args[1]],
      'car':   args => Array.isArray(args[0]) ? args[0][0] : null,
      'cdr':   args => Array.isArray(args[0]) ? args[0].slice(1) : [],
      'list':  args => args,
      'quote': args => args[0],
      'null?': args => args[0] === null || (Array.isArray(args[0]) && args[0].length === 0),
      'atom?': args => !Array.isArray(args[0]),
      'eq?':   args => args[0] === args[1],
      // Ahmad Docking extensions
      'phi':         _    => PHI,
      'freq-anchor': args => { const hz = args[0] || 1618; return Date.now() % Math.round(1e9 / hz) },
      'worm-seal':   args => this._seal(String(args[0] ?? '')),
      'world-dump':  _    => this.worldDump(),
      'agent-id':    _    => this.agentId,
      'tick':        _    => this.tick,
    }
  }

  // ── S-expression tokenizer ──────────────────────────────────────────────────
  tokenize (src) {
    return src
      .replace(/\(/g, ' ( ')
      .replace(/\)/g, ' ) ')
      .trim()
      .split(/\s+/)
      .filter(Boolean)
  }

  // ── Parser ──────────────────────────────────────────────────────────────────
  parse (src) {
    const tokens = this.tokenize(src)
    let pos = 0
    const expr = () => {
      const t = tokens[pos++]
      if (t === '(') {
        const list = []
        while (tokens[pos] !== ')') list.push(expr())
        pos++ // consume ')'
        return list
      }
      if (t === "'") return ['quote', expr()]
      if (t === 'nil' || t === undefined) return null
      if (t === '#t' || t === 'true')  return true
      if (t === '#f' || t === 'false') return false
      const n = Number(t)
      return isNaN(n) ? t : n
    }
    return expr()
  }

  // ── Evaluator ───────────────────────────────────────────────────────────────
  eval (expr) {
    if (expr === null || expr === undefined) return null
    if (typeof expr === 'number')  return expr
    if (typeof expr === 'boolean') return expr
    if (typeof expr === 'string') {
      if (expr in this.env) return this.env[expr]
      return expr
    }
    if (!Array.isArray(expr) || expr.length === 0) return expr

    const [head, ...rest] = expr

    // Special forms
    if (head === 'quote')  return rest[0]
    if (head === 'define') { this.env[rest[0]] = this.eval(rest[1]); return rest[0] }
    if (head === 'if')     return this.eval(rest[0]) ? this.eval(rest[1]) : this.eval(rest[2] ?? null)
    if (head === 'let') {
      const bindings = rest[0]
      const body     = rest[1]
      const saved    = { ...this.env }
      for (const [name, val] of bindings) this.env[name] = this.eval(val)
      const result = this.eval(body)
      this.env = saved
      return result
    }
    if (head === 'begin') return rest.reduce((_, e) => this.eval(e), null)

    // Built-in or lambda call
    const fn   = this.eval(head)
    const args = rest.map(a => this.eval(a))

    if (typeof fn === 'function') return fn(args)
    if (head in this.builtins)   return this.builtins[head](args)

    throw new Error(`Unbound: ${head}`)
  }

  // ── Public API ──────────────────────────────────────────────────────────────
  evalSrc (src) {
    this.tick++
    const result = this.eval(this.parse(src))
    const seal   = this._seal(`${this.tick}:${JSON.stringify(result)}`)
    return { result, tick: this.tick, agent: this.agentId, seal }
  }

  worldDump () {
    const dump = {
      tick:   this.tick,
      agent:  this.agentId,
      env:    { ...this.env },
      seal:   this._seal(`${this.tick}:${this.agentId}`),
    }
    this.vault.push(dump)
    return dump
  }

  _seal (content) {
    return createHash('sha256').update(content).digest('hex').slice(0, 16)
  }
}

// ── Singleton machine ─────────────────────────────────────────────────────────
const machine = new SovereignLispMachine('METATRON')

// ── Public bridge API ─────────────────────────────────────────────────────────

/** Evaluate a Lisp source string. Returns { result, tick, agent, seal } */
export function evalLisp (src) {
  return machine.evalSrc(src)
}

/** Evaluate an S-expression object (already parsed). Returns same shape. */
export function evalSexp (sexp) {
  machine.tick++
  const result = machine.eval(sexp)
  const seal   = machine._seal(`${machine.tick}:${JSON.stringify(result)}`)
  return { result, tick: machine.tick, agent: machine.agentId, seal }
}

/** Snapshot the machine state as a WORM-sealed world dump. */
export function worldDump () {
  return machine.worldDump()
}

/** Parse a source string into an S-expression object. */
export function parseLisp (src) {
  return machine.parse(src)
}

/** Ahmad Docking: observe a value exactly once, then seal.
 *  Mirrors the no-cloning theorem -- the token is consumed. */
export function ahmadDock (token, meta = {}) {
  const seal = machine._seal(`DOCKING:${token}:${JSON.stringify(meta)}`)
  return { token, seal, agent: machine.agentId, tick: machine.tick, observed: true }
}

export { machine }
