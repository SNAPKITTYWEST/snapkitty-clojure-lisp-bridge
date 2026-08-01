// funtan/deed_validator_bridge.mjs
//
// Funtan DSL -- JavaScript bridge for browser and Node.js.
// Parses deed-rules.lisp and evaluates trust deed claims without
// requiring the Haskell subprocess (useful in browser/Pages environments).
//
// For production server use: deed_validator.hs (full LiquidHaskell enforcement).
// For browser/browser-compatible use: this file.
//
// Ahmad Ali Parr -- Bel Esprit D'Accord Irrevocable Trust -- EIN 42-697643

// ── Minimal S-expression parser (mirrors deed_validator.hs parseSExprs) ───────

function stripComments(src) {
  return src.split('\n')
    .map(line => line.replace(/;;.*$/, ''))
    .join('\n')
}

function tokenize(src) {
  const tokens = []
  let i = 0
  const s = stripComments(src)
  while (i < s.length) {
    if (/\s/.test(s[i])) { i++; continue }
    if (s[i] === '(')  { tokens.push('('); i++; continue }
    if (s[i] === ')')  { tokens.push(')'); i++; continue }
    if (s[i] === '"') {
      let j = i + 1
      while (j < s.length && s[j] !== '"') j++
      tokens.push(s.slice(i, j + 1))
      i = j + 1
      continue
    }
    let j = i
    while (j < s.length && !/[\s()]/.test(s[j])) j++
    tokens.push(s.slice(i, j))
    i = j
  }
  return tokens
}

function parseExpr(tokens, pos) {
  if (pos >= tokens.length) return [null, pos]
  if (tokens[pos] === '(') {
    const list = []
    pos++
    while (pos < tokens.length && tokens[pos] !== ')') {
      const [expr, newPos] = parseExpr(tokens, pos)
      list.push(expr)
      pos = newPos
    }
    return [list, pos + 1]
  }
  const tok = tokens[pos]
  if (tok.startsWith('"')) return [tok.slice(1, -1), pos + 1]
  const n = Number(tok)
  if (!isNaN(n)) return [n, pos + 1]
  if (tok === 'true')  return [true,  pos + 1]
  if (tok === 'false') return [false, pos + 1]
  return [tok, pos + 1]
}

function parseFuntan(src) {
  const tokens = tokenize(src)
  const exprs  = []
  let pos = 0
  while (pos < tokens.length) {
    const [expr, newPos] = parseExpr(tokens, pos)
    if (expr !== null) exprs.push(expr)
    pos = newPos
  }
  return exprs
}

// ── Extract deed-spec fields ──────────────────────────────────────────────────

function extractSpec(exprs) {
  const spec = {}
  for (const expr of exprs) {
    if (!Array.isArray(expr) || expr[0] !== 'deed-spec') continue
    for (const field of expr.slice(1)) {
      if (!Array.isArray(field)) continue
      const [key, ...vals] = field
      spec[key] = vals.length === 1 ? vals[0] : vals
    }
  }
  return spec
}

// ── Validate a trust deed against a Funtan spec ───────────────────────────────

export function validateDeed(spec, deed) {
  const errors = []

  // trust-score-min / trust-score-max
  if (spec['trust-score-min'] !== undefined && deed.trustScore < spec['trust-score-min'])
    errors.push(`trust_score ${deed.trustScore} below minimum ${spec['trust-score-min']}`)
  if (spec['trust-score-max'] !== undefined && deed.trustScore > spec['trust-score-max'])
    errors.push(`trust_score ${deed.trustScore} above maximum ${spec['trust-score-max']}`)

  // seal-min-length
  if (spec['seal-min-length'] !== undefined && (!deed.seal || deed.seal.length < spec['seal-min-length']))
    errors.push(`seal too short: ${deed.seal?.length ?? 0} < ${spec['seal-min-length']}`)

  // seal-must-cover
  if (Array.isArray(spec['seal-must-cover'])) {
    for (const field of spec['seal-must-cover']) {
      if (deed[field] === undefined && deed.sealCovers && !deed.sealCovers.includes(field))
        errors.push(`seal must cover field: ${field}`)
    }
  }

  // expiry-required
  if (spec['expiry-required'] === true && !deed.expiresAt)
    errors.push('expiry_required but expiresAt not set')

  // globally-blocked-actions
  const blocked = Array.isArray(spec['globally-blocked-actions'])
    ? spec['globally-blocked-actions']
    : spec['globally-blocked-actions'] ? [spec['globally-blocked-actions']] : []
  if (deed.allowedActions) {
    for (const action of deed.allowedActions) {
      if (blocked.includes(action))
        errors.push(`globally blocked action in allowed-actions: ${action}`)
    }
  }

  // authority-min-trust
  if (spec['authority-min-trust'] !== undefined && deed.isIssuer) {
    if (deed.trustScore < spec['authority-min-trust'])
      errors.push(`issuer trust ${deed.trustScore} below authority-min-trust ${spec['authority-min-trust']}`)
  }

  // status-values
  const validStatuses = Array.isArray(spec['status-values']) ? spec['status-values'] : []
  if (validStatuses.length > 0 && deed.status && !validStatuses.includes(deed.status))
    errors.push(`invalid status: ${deed.status}. Must be one of: ${validStatuses.join(', ')}`)

  return {
    valid:  errors.length === 0,
    errors,
    spec,
    deed,
  }
}

// ── Main API ──────────────────────────────────────────────────────────────────

export function loadFuntanSpec(src) {
  const exprs = parseFuntan(src)
  return extractSpec(exprs)
}

// ── Example usage ─────────────────────────────────────────────────────────────

export const EXAMPLE_DEED = {
  agentId:        'METATRON',
  trustScore:     1.0,
  seal:           '0'.repeat(64),
  expiresAt:      Date.now() + 86400000,
  status:         'active',
  allowedActions: ['read', 'write', 'seal'],
  sealCovers:     ['agent-id', 'trust-score', 'expires-at', 'status',
                   'escalation-authority', 'allowed-actions', 'restricted-actions'],
}

if (typeof process !== 'undefined' && process.argv[1]?.endsWith('deed_validator_bridge.mjs')) {
  const { readFileSync } = await import('fs')
  const { join, dirname } = await import('path')
  const { fileURLToPath } = await import('url')

  const __dir = dirname(fileURLToPath(import.meta.url))
  const specSrc = readFileSync(join(__dir, 'deed-rules.lisp'), 'utf8')
  const spec    = loadFuntanSpec(specSrc)

  console.log('=== Funtan Spec Loaded ===')
  console.log(JSON.stringify(spec, null, 2))

  console.log('\n=== Validating example deed ===')
  const result = validateDeed(spec, EXAMPLE_DEED)
  console.log('Valid:', result.valid)
  if (!result.valid) console.log('Errors:', result.errors)
  else console.log('All checks passed.')
}
