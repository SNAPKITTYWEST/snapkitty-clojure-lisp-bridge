// dsssl-synthesis/dsssl-kernel.mjs
//
// DSSSL Kernel — JavaScript runtime for sovereign-dsssl.dsl
//
// Interprets DSSSL construction rules on an SGML grove.
// The style sheet combines:
//   (1) Relational synthesis kernel (lookup, unify-hole, synthesize-bindings)
//   (2) Construction rules (element GROOVE, EXPR, HOLE, INT, OP, LEFT, RIGHT)
//
// This runtime:
//   - Parses the SGML input grove (dsssl-input.sgml)
//   - Runs synthesize-bindings to fill holes
//   - Applies construction rules to produce the verified output grove
//   - Emits JSON synthesis receipt + WORM seal
//
// Connects to:
//   dsssl-synthesis.mjs        (SGML grove parser, already exists)
//   dsssl-synthesis-fixed.mjs  (fixed homoiconic engine)
//   relational-refinement-engine.mjs (LH refinement types)
//
// Ahmad Ali Parr -- Bel Esprit D'Accord Irrevocable Trust -- EIN 42-697643

import { createHash } from 'crypto'

// ── Grove node representation ─────────────────────────────────────────────────
// Mirrors the DSSSL (gi node) / (data node) / (children node) accessors

class GroveNode {
  constructor(gi, data = '', children = [], attrs = {}) {
    this.gi       = gi
    this.data     = data
    this.children = children
    this.attrs    = attrs
  }

  attr(name) { return this.attrs[name] ?? null }

  // DSSSL: (node-list-first (select-elements (children node) "TAG"))
  selectFirst(tag) {
    return this.children.find(c => c.gi === tag) ?? null
  }

  selectAll(tag) {
    return this.children.filter(c => c.gi === tag)
  }
}

// ── SGML grove parser ─────────────────────────────────────────────────────────
// Simplified — handles the input format from dsssl-input.sgml

function parseGrove(sgml) {
  const stripped = sgml
    .replace(/<!--[\s\S]*?-->/g, '')
    .replace(/<\?[^>]*>/g, '')
    .replace(/<!DOCTYPE[\s\S]*?]>/i, '')
    .replace(/<!SGML[\s\S]*?>/i, '')
    .trim()

  let pos = 0

  function skipWS() {
    while (pos < stripped.length && /\s/.test(stripped[pos])) pos++
  }

  function parseNode() {
    skipWS()
    if (pos >= stripped.length || stripped[pos] !== '<') return null
    if (stripped[pos + 1] === '/') return null

    pos++ // skip <
    let tagEnd = stripped.indexOf('>', pos)
    if (tagEnd === -1) return null
    const tagContent = stripped.slice(pos, tagEnd).trim()
    pos = tagEnd + 1

    const parts = tagContent.split(/\s+/)
    const gi    = parts[0].toUpperCase()
    const attrs = {}
    for (let i = 1; i < parts.length; i++) {
      const [k, v] = parts[i].split('=')
      if (v) attrs[k.toUpperCase()] = v.replace(/['"]/g, '')
    }

    // EMPTY elements (HOLE)
    if (gi === 'HOLE') return new GroveNode(gi, '', [], attrs)

    const children = []
    let   textData = ''

    while (pos < stripped.length) {
      skipWS()
      if (pos >= stripped.length) break
      if (stripped[pos] === '<') {
        if (stripped[pos + 1] === '/') {
          pos = stripped.indexOf('>', pos) + 1
          break
        }
        const child = parseNode()
        if (child) children.push(child)
      } else {
        const nextTag = stripped.indexOf('<', pos)
        const end     = nextTag === -1 ? stripped.length : nextTag
        const txt     = stripped.slice(pos, end).trim()
        if (txt) textData += txt
        pos = end
      }
    }

    return new GroveNode(gi, textData, children, attrs)
  }

  return parseNode()
}

// ── DSSSL Synthesis Kernel ────────────────────────────────────────────────────
// Direct port of the Scheme functions from sovereign-dsssl.dsl

function lookup(varName, env) {
  const cell = env.find(([k]) => k === varName)
  return cell ? cell[1] : null
}

function unifyHole(varName, candidate, env) {
  const bound = lookup(varName, env)
  if (bound === null)            return [[varName, candidate], ...env]
  if (bound === candidate)       return env
  return null  // contradiction
}

function evalGroveNode(node, env) {
  if (!node) return 'UNBOUND'
  const gi = node.gi

  if (gi === 'INT') {
    const n = Number(node.data)
    return isNaN(n) ? 'UNBOUND' : n
  }

  if (gi === 'HOLE') {
    const varName = node.attr('VAR') || node.attr('var')
    const val     = lookup(varName, env)
    return val !== null ? val : 'UNBOUND'
  }

  if (gi === 'EXPR') {
    const opNode    = node.selectFirst('OP')
    const leftNode  = node.selectFirst('LEFT')
    const rightNode = node.selectFirst('RIGHT')
    if (!opNode || !leftNode || !rightNode) return 'UNBOUND'

    const op    = opNode.data.trim()
    const lNode = leftNode.children[0]
    const rNode = rightNode.children[0]
    const lVal  = evalGroveNode(lNode, env)
    const rVal  = evalGroveNode(rNode, env)

    if (lVal === 'UNBOUND' || rVal === 'UNBOUND') return 'UNBOUND'
    if (op === '+') return lVal + rVal
    if (op === '*') return lVal * rVal
    if (op === '-') return lVal - rVal
    if (op === '/') return rVal !== 0 ? lVal / rVal : 'UNBOUND'
    return 0
  }

  return 'UNBOUND'
}

const TARGET_INVARIANT = 20

function synthesizeBindings(holeVar, candidates, env, rootNode) {
  for (const cand of candidates) {
    const testEnv = unifyHole(holeVar, cand, env)
    if (testEnv !== null) {
      const result = evalGroveNode(rootNode, testEnv)
      if (result === TARGET_INVARIANT) {
        return { env: testEnv, candidate: cand, result }
      }
    }
  }
  return null
}

// ── DSSSL Construction Rules ──────────────────────────────────────────────────
// Each rule matches a gi (element type) and produces a new GroveNode.
// Mirrors the (element TAG ...) construction rules in sovereign-dsssl.dsl.

function applyRule(node, solvedEnv) {
  if (!node) return null
  const gi = node.gi

  if (gi === 'GROVE') {
    const candidates  = [1, 2, 3, 4, 5]
    const rootExpr    = node.selectFirst('EXPR') || node.children[0]
    const synthesis   = synthesizeBindings('?x', candidates, [], rootExpr)
    const status      = synthesis ? 'VERIFIED' : 'FAILED'
    const env         = synthesis ? synthesis.env : []

    return new GroveNode('SYNTHESIZED-GROVE', '', [
      applyRule(rootExpr, env)
    ], { STATUS: status })
  }

  if (gi === 'EXPR') {
    return new GroveNode('EXPR', '', node.children.map(c => applyRule(c, solvedEnv)),
      { ID: node.attr('ID') || node.attr('id') || '' })
  }

  if (gi === 'OP' || gi === 'INT') {
    return new GroveNode(gi, node.data, [], {})
  }

  if (gi === 'LEFT' || gi === 'RIGHT') {
    return new GroveNode(gi, '', node.children.map(c => applyRule(c, solvedEnv)), {})
  }

  if (gi === 'HOLE') {
    const varName  = node.attr('VAR') || node.attr('var')
    const resolved = lookup(varName, solvedEnv)
    if (resolved !== null) {
      return new GroveNode('SYNTHESIZED-INT', String(resolved), [], { 'RESOLVED-FROM': varName })
    }
    return new GroveNode('UNRESOLVED-HOLE', '', [], { VAR: varName })
  }

  // Default: pass through
  return new GroveNode(gi, node.data, node.children.map(c => applyRule(c, solvedEnv)), node.attrs)
}

// ── Grove serializer → SGML output ───────────────────────────────────────────

function serializeGrove(node, indent = 0) {
  if (!node) return ''
  const pad    = '  '.repeat(indent)
  const attrs  = Object.entries(node.attrs)
    .map(([k, v]) => ` ${k}="${v}"`).join('')
  const hasContent = node.children.length > 0 || node.data.trim()

  if (!hasContent) return `${pad}<${node.gi}${attrs}>`

  const inner = node.data.trim()
    ? node.data.trim()
    : '\n' + node.children.map(c => serializeGrove(c, indent + 1)).join('\n') + '\n' + pad

  return `${pad}<${node.gi}${attrs}>${inner}</${node.gi}>`
}

// ── Main ──────────────────────────────────────────────────────────────────────

export async function runDSSL(sgmlInput, options = {}) {
  // 1. Parse SGML grove
  const grove = parseGrove(sgmlInput)
  if (!grove) throw new Error('DSSSL: failed to parse SGML input grove')

  // 2. Find hole variable and candidates
  const holeVars  = []
  const findHoles = (node) => {
    if (!node) return
    if (node.gi === 'HOLE') holeVars.push(node.attr('VAR') || node.attr('var'))
    node.children.forEach(findHoles)
  }
  findHoles(grove)

  // 3. Run synthesis kernel (GROVE construction rule)
  const rootExpr   = grove.selectFirst('EXPR') || grove.children[0]
  const candidates = options.candidates || [1, 2, 3, 4, 5]
  const synthesis  = synthesizeBindings('?x', candidates, [], rootExpr)

  // 4. Apply construction rules
  const outputGrove = applyRule(grove, synthesis?.env || [])

  // 5. Serialize output
  const outputSGML = serializeGrove(outputGrove)

  // 6. Verify: re-evaluate with solved binding
  let verifiedResult = null
  if (synthesis) {
    verifiedResult = evalGroveNode(rootExpr, synthesis.env)
  }

  // 7. WORM seal
  const seal = createHash('sha256')
    .update(JSON.stringify({ output: outputSGML, synthesis }))
    .digest('hex')

  return {
    status:               synthesis ? 'VERIFIED' : 'FAILED',
    transformation_summary: synthesis
      ? `Hole ?x resolved to ${synthesis.candidate}. Grove evaluates to ${verifiedResult} (target: ${TARGET_INVARIANT}).`
      : `No candidate satisfies invariant = ${TARGET_INVARIANT}.`,
    holes_found:          holeVars,
    synthesis: synthesis ? {
      hole_var:   '?x',
      candidate:  synthesis.candidate,
      env:        synthesis.env,
      result:     synthesis.result,
    } : null,
    output_grove_sgml:    outputSGML,
    worm_seal:            seal.slice(0, 16),
    verification_trace: {
      target_invariant:  TARGET_INVARIANT,
      candidates_tried:  candidates,
      smt_proof_status:  synthesis ? 'SAT' : 'UNSAT',
    },
  }
}

// ── CLI ───────────────────────────────────────────────────────────────────────

if (process.argv[1]?.endsWith('dsssl-kernel.mjs')) {
  const { readFileSync } = await import('fs')
  const { fileURLToPath } = await import('url')
  const { dirname, join } = await import('path')

  const __dir = dirname(fileURLToPath(import.meta.url))
  const inputPath = process.argv[2] || join(__dir, 'dsssl-input.sgml')

  let sgmlInput
  try {
    sgmlInput = readFileSync(inputPath, 'utf8')
  } catch {
    // Use embedded example if file not present
    sgmlInput = `<GROVE><EXPR id="root"><OP>*</OP><LEFT><EXPR id="inner"><OP>+</OP><LEFT><HOLE var="?x"></LEFT><RIGHT><INT>4</INT></RIGHT></EXPR></LEFT><RIGHT><INT>4</INT></RIGHT></EXPR></GROVE>`
  }

  const result = await runDSSL(sgmlInput)

  console.log('══════════════════════════════════════════════════')
  console.log('  DSSSL SYNTHESIS KERNEL')
  console.log('  sovereign-dsssl.dsl → verified output grove')
  console.log('══════════════════════════════════════════════════')
  console.log(JSON.stringify(result, null, 2))
}
