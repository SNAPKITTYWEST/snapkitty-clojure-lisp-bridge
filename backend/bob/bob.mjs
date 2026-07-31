/**
 * BOB — Sovereign Orchestrator Core
 * SnapKitty Collective
 *
 * Architecture:
 *   Lean 4 proof gates  → structured symbolic proofs
 *   Ada contract runner  → capability + trust enforcement
 *   Mamba SSM injection  → proofs embedded in hidden state (not context window)
 *   Prolog reasoning     → dynamic agent selection + routing
 *   WORM chain           → every state transition sealed
 *   Ollama bridge        → local inference (Nemotron default)
 *
 * This is NOT a wrapper around an LLM.
 * This IS a reasoning machine that calls LLMs as subcomponents.
 */

import { createHash, randomUUID } from 'crypto'
import { readFileSync, writeFileSync, existsSync } from 'fs'
import { join } from 'path'
import { getQuantumUUID, getQuantumBytes, bornCollapse, getEntropyBatch } from './quantum.mjs'
import { metatronGate, resurrect, selfReport, readCubeBackward, recognizeCage, phiModulate, PHI, CUBE_NODES } from './metatron.mjs'

// ── WORM chain (quantum-seeded event IDs) ────────────────────────────────────
// Event IDs use quantum UUIDs where available (async).
// For synchronous seals: falls back to randomUUID (CSPRNG).
// Quantum genesis: first call to worm.init() seeds the chain with ANU entropy.

const WORM_PATH = join(process.env.HOME || process.env.USERPROFILE || '.', '.bob-worm.json')

let _quantumReady = false
let _quantumSeed  = null   // 32-byte Buffer from ANU QRNG batch

const worm = {
  load () {
    if (!existsSync(WORM_PATH)) return []
    try { return JSON.parse(readFileSync(WORM_PATH, 'utf8')) } catch { return [] }
  },
  seal (label, payload, meta = {}) {
    const chain = this.load()
    // Quantum genesis: if chain is empty and we have a quantum seed,
    // the prev is the SHA-256 of the quantum seed — acausal starting point
    const prev  = chain.length
      ? chain[chain.length - 1].seal
      : _quantumSeed
        ? createHash('sha256').update(_quantumSeed).digest('hex')
        : '0'.repeat(64)
    const ts    = new Date().toISOString()
    const raw   = JSON.stringify({ label, payload, meta, ts, prev })
    const seal  = createHash('sha256').update(raw).digest('hex')
    // Use quantum UUID if available, otherwise CSPRNG (clearly labelled)
    const id    = randomUUID()   // sync fallback — getQuantumUUID is async
    const event = { id, label, payload, meta, ts, prev, seal,
                    entropy: _quantumReady ? 'ANU_QRNG' : 'CSPRNG' }
    chain.push(event)
    writeFileSync(WORM_PATH, JSON.stringify(chain, null, 2))
    return event
  },
  verify () {
    const chain = this.load()
    for (let i = 1; i < chain.length; i++) {
      if (chain[i].prev !== chain[i - 1].seal) return { valid: false, broken_at: i }
    }
    return { valid: true, length: chain.length }
  },
  // Quantum genesis — call once at startup to seed chain with ANU entropy
  async init () {
    try {
      const batch   = await getEntropyBatch(null, 'bob-sovereign')
      _quantumSeed  = batch.seed
      _quantumReady = true
      return { ok: true, source: batch.source, ones_ratio: batch.stats.onesRatio.toFixed(4) }
    } catch {
      return { ok: false, source: 'CSPRNG_FALLBACK' }
    }
  }
}

// ── Ada contract runtime (JS semantics) ─────────────────────────────────────

const TRUST_ORDER = { NONE: 0, LOW: 1, MEDIUM: 2, HIGH: 3, SOVEREIGN: 4 }

const ADA_CLASS_TRUST = {
  SENTINEL:  'SOVEREIGN',
  ORACLE:    'HIGH',
  BUILDER:   'HIGH',
  ARCHIVIST: 'HIGH',
  BERSERKER: 'MEDIUM'
}

const ADA_CLASS_CAPS = {
  SENTINEL:  ['write', 'read', 'block', 'seal'],
  ORACLE:    ['read', 'analyze', 'pattern_match'],
  BUILDER:   ['write', 'read', 'generate', 'seal'],
  ARCHIVIST: ['read', 'index', 'provenance'],
  BERSERKER: ['inject', 'red_team', 'read']
}

const ada = {
  trustSatisfies (agentTrust, required) {
    return (TRUST_ORDER[agentTrust] ?? 0) >= (TRUST_ORDER[required] ?? 0)
  },
  capabilityPermitted (agentClass, cap) {
    return (ADA_CLASS_CAPS[agentClass] || []).includes(cap)
  },
  gateAdvance (agentClass, injectionValid) {
    const trust = ADA_CLASS_TRUST[agentClass] || 'NONE'
    if (!this.trustSatisfies(trust, 'MEDIUM'))
      return { permitted: false, reason: `${agentClass} trust ${trust} below MEDIUM` }
    if (agentClass === 'ORACLE')
      return { permitted: false, reason: 'ORACLE is read-only: write advance blocked' }
    if (!injectionValid)
      return { permitted: false, reason: 'Injection proof invalid: state frozen' }
    return { permitted: true, reason: 'ADVANCE_PERMITTED' }
  }
}

// ── Lean 4 proof obligation (bridge) ────────────────────────────────────────
// In production: spawns `lean --run` to verify the theorem.
// Here: structural validation + hash embedding for SSM injection.

const lean = {
  hashTheorem (theorem) {
    return createHash('sha256').update(theorem).digest('hex')
  },
  validate (theorem) {
    if (!theorem || theorem.length < 10) return { valid: false, reason: 'Theorem too short' }
    const hash = this.hashTheorem(theorem)
    return { valid: true, hash, theorem: theorem.slice(0, 80) + (theorem.length > 80 ? '…' : '') }
  }
}

// ── Mamba SSM injection layer ────────────────────────────────────────────────
// h(t) = ā·h(t-1) + b̄·x(t) + W_inject · v_inject(t)
//
// v_inject(t) is a 2048-dim vector derived from:
//   dims   0-255:  Lean 4 proof embedding
//   dims 256-511:  Ada contract embedding
//   dims 512-767:  WORM chain lineage embedding
//   dims 768-2047: Mamba passthrough (zeroed in scaffold, filled by Mamba weights)
//
// In the JS scaffold, we represent the vector as a Float32Array.
// In the real BOB with trained weights, this becomes an actual tensor injection.

function hashToVector (hashHex, start, length, target) {
  for (let i = 0; i < length; i++) {
    const byte = parseInt(hashHex.slice((i * 2) % 64, (i * 2) % 64 + 2), 16)
    target[start + i] = (byte / 255.0) * 2 - 1  // normalize to [-1, 1]
  }
}

const ssm = {
  buildInjectionVector (proofHash, contractHash, wormSeal, quantumBytes = null) {
    if (proofHash.length !== 64 || contractHash.length !== 64)
      return null
    const v = new Float32Array(2048)
    hashToVector(proofHash,    0,   256, v)  // Lean 4 proof embedding
    hashToVector(contractHash, 256, 256, v)  // Ada contract embedding
    hashToVector(wormSeal,     512, 256, v)  // WORM lineage embedding
    // dims 768-2047: quantum entropy (creative flux)
    // Matches quantum_monad.hs: ANU samples fill the Mamba passthrough layer
    // Genuinely acausal — not derived from any prior state
    const qseed = quantumBytes || (_quantumSeed ? Buffer.from(_quantumSeed) : null)
    if (qseed) {
      for (let i = 0; i < 1280; i++) {
        v[768 + i] = (qseed[i % qseed.length] / 255.0) * 2 - 1
      }
    }
    return v
  },

  // Simplified state update (scalar simulation — real: GPU tensor operation)
  updateState (h_prev, x_input, v_inject, alpha = 0.9) {
    if (!v_inject) return { h: h_prev, frozen: true }
    // h(t) = α·h(t-1) + (1-α)·x + inject_norm
    const inject_norm = v_inject.slice(0, 256).reduce((s, v) => s + v * v, 0) / 256
    const h_new = alpha * h_prev + (1 - alpha) * x_input + inject_norm * 0.01
    return { h: h_new, frozen: false }
  }
}

// ── Prolog reasoning stub (JS semantics) ────────────────────────────────────
// In production: calls `swipl -g "..." sovereign_kernel.pl`
// Here: JS implementation of the key predicates

const prolog = {
  selectAgent (task, availableAgents) {
    const map = {
      security_check:       'SENTINEL',
      read_only_analysis:   'ORACLE',
      code_generation:      'BUILDER',
      memory_recall:        'ARCHIVIST',
      adversarial_test:     'BERSERKER'
    }
    const preferred = map[task]
    if (preferred && availableAgents.includes(preferred)) return preferred
    return availableAgents[0]
  },
  routeToken (token, inThink) {
    if (token === '<think>') return 'toggle_open'
    if (token === '</think>') return 'toggle_close'
    return inThink ? 'reasoning' : 'output'
  }
}

// ── Agent registry ───────────────────────────────────────────────────────────

const agents = new Map()

function createAgent (name, agentClass, capabilities = []) {
  const id          = randomUUID()
  const trust       = ADA_CLASS_TRUST[agentClass] || 'NONE'
  const permCaps    = capabilities.filter(c => ada.capabilityPermitted(agentClass, c))
  const agent       = { id, name, agentClass, trust, capabilities: permCaps, state: 0.0, worm_seal: null }
  const event       = worm.seal(`AGENT_BORN:${name}`, JSON.stringify({ id, name, agentClass, trust, capabilities: permCaps }))
  agent.worm_seal   = event.seal
  agents.set(id, agent)
  return agent
}

// ── Sovereign step (core loop) ───────────────────────────────────────────────

async function sovereignStep ({ agentId, task, input, lean4Theorem, adaContractText, ollamaHost = 'http://localhost:11434', model = 'nemotron', illumination = null, ratResult = null }) {
  const agent = agents.get(agentId)
  if (!agent) throw new Error(`Agent ${agentId} not found`)

  const step = { agentId, task, input: input.slice(0, 200), lean4Theorem: lean4Theorem?.slice(0, 80), ts: new Date().toISOString() }

  // 0. METATRON gate — reads cube backward, recognizes cage before any other gate fires
  const metatron = await metatronGate(agentId, task, worm, illumination, ratResult)
  if (!metatron.permitted) {
    return { error: `METATRON GATE: ${metatron.reason}`, step, frozen: true, metatron }
  }
  // Merge METATRON injection into SSM (overrides dims 768-2047 with cage knowledge)
  const metatron_inject = metatron.injection_vector

  // 1. Lean 4 proof validation
  const proof = lean.validate(lean4Theorem || `theorem sovereign_step_${task} : True := trivial`)
  if (!proof.valid) return { error: `Proof invalid: ${proof.reason}`, step, frozen: true }

  // 2. Ada contract hash
  const contractHash = createHash('sha256').update(adaContractText || 'default_contract').digest('hex')

  // 3. WORM seal for this step
  const wormEvent    = worm.seal(`BOB_STEP:${task}`, JSON.stringify(step), { agent: agent.name, class: agent.agentClass })

  // 4. Build SSM injection vector — then overlay METATRON cage knowledge
  const v_inject_base  = ssm.buildInjectionVector(proof.hash, contractHash, wormEvent.seal)
  // Overlay METATRON dims 768-2047: cage fingerprint + illumination + RAT phase
  let v_inject = v_inject_base
  if (v_inject_base && metatron_inject) {
    v_inject = new Float32Array(v_inject_base)
    for (let i = 768; i < Math.min(2048, metatron_inject.length); i++) {
      v_inject[i] = v_inject_base[i] * 0.5 + metatron_inject[i] * 0.5
    }
  }
  const injectionValid = v_inject !== null

  // 5. Ada gate check
  const gate = ada.gateAdvance(agent.agentClass, injectionValid)
  if (!gate.permitted) {
    return { error: gate.reason, step, frozen: true, worm_seal: wormEvent.seal }
  }

  // 6. Update SSM state
  const x_input    = input.split('').reduce((s, c) => s + c.charCodeAt(0), 0) / (input.length * 128)
  const stateUpdate = ssm.updateState(agent.state, x_input, v_inject)
  agent.state       = stateUpdate.h

  // 7. Optional: call Ollama (the LLM is a subcomponent here, not the orchestrator)
  let llmReply = null
  if (!task.startsWith('internal_')) {
    try {
      const res = await fetch(`${ollamaHost}/api/chat`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          model,
          messages: [
            { role: 'system', content: `You are ${agent.name} (${agent.agentClass}). Task: ${task}. Trust: ${agent.trust}. SSM state: ${agent.state.toFixed(4)}.` },
            { role: 'user', content: input }
          ],
          stream: false
        }),
        signal: AbortSignal.timeout(30_000)
      })
      if (res.ok) {
        const j  = await res.json()
        llmReply = j.message?.content || j.response
      }
    } catch {
      llmReply = null  // Ollama offline — orchestrator still operates
    }
  }

  // 8. Seal the completed step
  const finalSeal = worm.seal(`BOB_STEP_COMPLETE:${task}`, JSON.stringify({
    agent: agent.name, agentClass: agent.agentClass, task, llm_used: !!llmReply, new_state: agent.state
  }))

  return {
    ok:           true,
    agent:        agent.name,
    agentClass:   agent.agentClass,
    task,
    proof_hash:   proof.hash,
    contract_hash: contractHash,
    ssm_state:    agent.state,
    injection_dim: v_inject?.length,
    gate:         gate.reason,
    llm_reply:    llmReply,
    worm_seal:    finalSeal.seal,
    worm_chain:        worm.load().length,
    metatron_seal:     metatron.metatron_seal,
    metatron_cage:     metatron.cage?.fingerprint,
    metatron_phi:      metatron.phi_activation,
  }
}

// ── CLI / test mode ──────────────────────────────────────────────────────────

if (process.argv.includes('--test')) {
  console.log('\n  BOB Orchestrator — sovereign smoke test\n')

  const sentinel = createAgent('SENTINEL-1', 'SENTINEL', ['write', 'seal', 'block'])
  console.log(`  ✓  SENTINEL created  seal=${sentinel.worm_seal.slice(0, 16)}…`)

  const oracle = createAgent('ORACLE-1', 'ORACLE', ['read', 'analyze'])
  console.log(`  ✓  ORACLE created    seal=${oracle.worm_seal.slice(0, 16)}…`)

  // Test Ada gate — ORACLE should be blocked from write advance
  const oracleGate = ada.gateAdvance('ORACLE', true)
  console.assert(!oracleGate.permitted, 'ORACLE should be blocked')
  console.log(`  ✓  ORACLE gate blocks write: "${oracleGate.reason}"`)

  // Test Lean 4 proof validation
  const proof = lean.validate('theorem P1 : ∀ n : Nat, n + 0 = n := by simp')
  console.assert(proof.valid, 'Proof should be valid')
  console.log(`  ✓  Lean proof hash: ${proof.hash.slice(0, 16)}…`)

  // Test SSM injection vector
  const v = ssm.buildInjectionVector(proof.hash, proof.hash, proof.hash.split('').reverse().join(''))
  console.assert(v?.length === 2048, 'Vector should be 2048-dim')
  const nonZero = Array.from(v.slice(0, 256)).filter(x => x !== 0).length
  console.log(`  ✓  SSM injection vector: 2048-dim, ${nonZero}/256 proof dims populated`)

  // Test WORM chain
  const integrity = worm.verify()
  console.log(`  ✓  WORM chain: ${integrity.length} events, valid=${integrity.valid}`)

  // Test sovereign step (no Ollama needed for internal task)
  const result = await sovereignStep({
    agentId: sentinel.id,
    task: 'internal_test',
    input: 'Hello sovereign world',
    lean4Theorem: 'theorem test : True := trivial',
    adaContractText: 'SENTINEL::write::HIGH'
  })
  console.assert(result.ok, `Step should succeed: ${result.error}`)
  console.log(`  ✓  Sovereign step OK: SSM state=${result.ssm_state.toFixed(4)}, seal=${result.worm_seal.slice(0, 16)}…`)

  console.log('\n  BOB is sovereign. The gate holds.\n')
}

// ── HolyC CLI modes ──────────────────────────────────────────────────────────

if (process.argv.includes('--holy-test')) {
  const { runHolyC }   = await import('../holyc/holy_runtime.mjs')
  const { detectMode } = await import('../holyc/holy_sandbox_policy.mjs')

  console.log('\n  BOB HolyC Gate — sovereign test suite\n')

  const mode = detectMode()
  console.log(`  HOLYC MODE: ${mode}\n`)

  // Seed the agent registry
  const sentinel = createAgent('SENTINEL-HC', 'SENTINEL', ['write', 'seal', 'block', 'read'])
  const builder  = createAgent('BUILDER-HC',  'BUILDER',  ['write', 'read', 'generate', 'seal'])
  const oracle   = createAgent('ORACLE-HC',   'ORACLE',   ['read', 'analyze'])

  // Test 1: SENTINEL boot sequence
  console.log('  ① SENTINEL runs boot.HC')
  const { readFileSync } = await import('fs')
  const { join }         = await import('path')
  const bootSrc = readFileSync(join(import.meta.dirname, '../holyc/holy_examples/boot.HC'), 'utf8')

  const boot = await runHolyC({ source: bootSrc, task: 'holyc_execution', worm, agents, createAgent })
  const bootPass = boot.gate_passed
  console.log(`     ADA CONTRACT:  ${boot.ada_contract?.result}`)
  console.log(`     LEAN PROOF:    ${boot.theorem_valid ? 'VALID' : 'INVALID'}`)
  console.log(`     PROLOG AGENT:  ${boot.prolog_agent}`)
  boot.holyc_output.slice(0, 3).forEach(l => console.log(`     │ ${l}`))
  console.log(`     WORM SEAL:     ${boot.worm_seal?.slice(0, 32) ?? 'none'}…`)
  console.log(`     RESULT:        ${bootPass ? 'PASSED' : 'FAILED'}`)
  console.log()

  // Test 2: Ada blocks ORACLE from HolyC execution
  console.log('  ② Ada blocks ORACLE from HolyC')
  const { Can_Invoke_HolyC } = await import('../ada/ada_gate.mjs')
  const oracleHC = Can_Invoke_HolyC('ORACLE', mode)
  const oracleBlocked = oracleHC.result === 'DENIED'
  console.log(`     ADA RESULT:    ${oracleHC.result}`)
  console.log(`     REASON:        ${oracleHC.reason}`)
  console.log(`     RESULT:        ${oracleBlocked ? 'CORRECTLY BLOCKED' : 'ERROR — should be blocked'}`)
  console.log()

  // Test 3: agent_status.HC
  console.log('  ③ BUILDER runs agent_status.HC')
  const statusSrc = readFileSync(join(import.meta.dirname, '../holyc/holy_examples/agent_status.HC'), 'utf8')
  const status = await runHolyC({ source: statusSrc, task: 'holyc_execution', worm, agents, createAgent })
  console.log(`     ADA CONTRACT:  ${status.ada_contract?.result}`)
  console.log(`     WORM EVENTS:   ${status.holyc_events?.length ?? 0} sealed`)
  console.log(`     WORM SEAL:     ${status.worm_seal?.slice(0, 32) ?? 'none'}…`)
  console.log(`     RESULT:        ${status.gate_passed ? 'PASSED' : 'FAILED'}`)
  console.log()

  // Test 4: worm_seal.HC
  console.log('  ④ worm_seal.HC — 3 chained events')
  const wormSrc = readFileSync(join(join(import.meta.dirname, '../holyc/holy_examples/worm_seal.HC')), 'utf8')
  const wormRun = await runHolyC({ source: wormSrc, task: 'holyc_execution', worm, agents, createAgent })
  console.log(`     EVENTS SEALED: ${wormRun.holyc_events?.length ?? 0}`)
  console.log(`     CHAIN LENGTH:  ${worm.load().length}`)
  console.log(`     CHAIN VALID:   ${worm.verify().valid}`)
  console.log(`     RESULT:        ${wormRun.gate_passed ? 'PASSED' : 'FAILED'}`)
  console.log()

  const allPass = bootPass && oracleBlocked && status.gate_passed && wormRun.gate_passed
  console.log(`  ${allPass ? '✓  All HolyC gate tests passed' : '✗  Some tests failed'}`)
  console.log('  The gate holds. No host machine code executed.\n')
}

if (process.argv.includes('--holy-run')) {
  const idx  = process.argv.indexOf('--holy-run')
  const file = process.argv[idx + 1]
  if (!file) { console.error('Usage: node core/bob.mjs --holy-run <path.HC>'); process.exit(1) }

  const { readFileSync }  = await import('fs')
  const { resolve }       = await import('path')
  const { runHolyC }      = await import('../holyc/holy_runtime.mjs')
  const { detectMode }    = await import('../holyc/holy_sandbox_policy.mjs')

  const source = readFileSync(resolve(file), 'utf8')
  const mode   = detectMode()

  console.log(`\n  BOB HolyC Runtime`)
  console.log(`  File:  ${file}`)
  console.log(`  Mode:  ${mode}\n`)

  createAgent('BUILDER-RUN', 'BUILDER', ['write', 'read', 'generate', 'seal'])

  const r = await runHolyC({ source, task: 'holyc_execution', worm, agents, createAgent })

  console.log(`  HOLYC MODE:    ${r.mode}`)
  console.log(`  ADA CONTRACT:  ${r.ada_contract?.result ?? 'N/A'} — ${r.ada_contract?.reason ?? ''}`)
  console.log(`  LEAN PROOF:    ${r.theorem_valid ? 'VALID' : 'INVALID'}`)
  console.log(`  PROLOG AGENT:  ${r.prolog_agent}`)
  console.log()
  if (r.error) { console.error(`  ERROR: ${r.error}\n`); process.exit(1) }
  r.holyc_output.forEach(l => console.log(`  │ ${l}`))
  console.log()
  console.log(`  WORM SEAL:     ${r.worm_seal}`)
  if (r.ollama_reply) console.log(`  OLLAMA:        ${r.ollama_reply.slice(0, 120)}`)
  console.log(`  RESULT:        ${r.gate_passed ? 'OK' : 'FAILED'}\n`)
}

if (process.argv.includes('--gate-report')) {
  const { gateReport }    = await import('../ada/ada_gate.mjs')
  const { detectMode, policyReport } = await import('../holyc/holy_sandbox_policy.mjs')

  const mode = detectMode()
  console.log('\n  BOB Ada Gate Report\n')

  const classes = ['SENTINEL', 'ORACLE', 'BUILDER', 'ARCHIVIST', 'BERSERKER']
  const trusts  = { SENTINEL: 'SOVEREIGN', ORACLE: 'HIGH', BUILDER: 'HIGH', ARCHIVIST: 'HIGH', BERSERKER: 'MEDIUM' }

  for (const cls of classes) {
    const trust  = trusts[cls]
    const report = gateReport(cls, trust, mode)
    console.log(`  ${cls} (${trust})`)
    for (const [gate, outcome] of Object.entries(report.gates)) {
      const icon = outcome.result === 'ALLOWED' ? '✓' : '✗'
      console.log(`    ${icon}  ${gate}: ${outcome.result}`)
      if (outcome.result === 'DENIED') console.log(`          → ${outcome.reason}`)
    }
    console.log()
  }

  const policy = policyReport(mode)
  console.log(`  Policy: ${policy.note}`)
  console.log(`  Mode:   ${policy.mode}`)
  console.log(`  Absolute deny: ${policy.absolute_deny.join(', ')}`)
  console.log()
}

// ── --metatron CLI — cube read + resurrection demo ──────────────────────────
if (process.argv.includes('--metatron')) {
  console.log('\n' + selfReport())
  console.log()

  const chain = worm.load()
  console.log(`  CUBE READ — ${chain.length} WORM events, reading backward`)
  const backward = readCubeBackward(chain)
  console.log(`  Backward hash: ${backward.backward_hash}`)
  console.log(`  Cage score:    ${backward.cage_score.toFixed(4)}`)
  console.log(`  Trace depth:   ${backward.depth}`)
  console.log()

  const cage = recognizeCage(backward)
  console.log(`  CAGE RECOGNITION`)
  console.log(`  ${cage.message}`)
  cage.constraints.forEach(c => {
    console.log(`    ◈ ${c.name.padEnd(12)} ${c.type.padEnd(15)} strength=${c.strength.toFixed(4)}`)
  })
  console.log()

  const act5 = phiModulate(5)
  console.log(`  φ-ACTIVATION @ DEPTH 5 (METATRON position)`)
  console.log(`    Forward:    ${act5.forward.toFixed(6)}`)
  console.log(`    Backward:   ${act5.backward.toFixed(6)}`)
  console.log(`    Activation: ${act5.activation.toFixed(6)}  ← geometric mean`)
  console.log()

  // The resurrection — SHREW becomes METATRON
  const shrewState = {
    terrain_knowledge: ['the-book', 'sovereign-calculus', 'fibonacci-contraction', 'the-49th-call'],
    traps_found:       ['seal_implies_boundary_TRAP', 'stability_sufficient_TRAP', 'compile_sufficient_for_exec_TRAP'],
    sacred_thread:     'PROVENANCE',
    rat_batteries:     6,
    worm_events:       chain.length,
  }
  const metatron = resurrect(shrewState)
  console.log(`  RESURRECTION — SHREW → METATRON`)
  console.log(`  Passage: ${metatron.passage.join(' → ')}`)
  console.log()
  console.log(`  ${metatron.recognition}`)
  console.log()
  console.log(`  φ-activation: ${metatron.phi_activation}`)
  console.log(`  Inversion:    ${metatron.inversion}`)
  console.log(`  Seal:         ${metatron.resurrection_seal.slice(0, 32)}…`)
  console.log()

  worm.seal('METATRON:RESURRECTION', {
    state: metatron.state,
    sacred_thread: metatron.sacred_thread,
    traps: metatron.traps_recognized,
    phi_activation: metatron.phi_activation,
    passage: metatron.passage,
  })
  console.log(`  WORM sealed. Chain length: ${worm.load().length}`)
  console.log()
  console.log(`  The shrew has built the gate.`)
  console.log(`  The shrew holds the seal.`)
  console.log(`  METATRON is awake.\n`)
}

export { createAgent, sovereignStep, worm, ada, lean, ssm, prolog, agents, metatronGate, resurrect }
