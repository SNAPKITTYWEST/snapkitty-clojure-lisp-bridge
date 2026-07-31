/**
 * ABZU IDE Bridge — WebSocket push to Phoenix LiveView
 *
 * The ABZU Sovereign BEAM IDE (Phoenix LiveView, Elixir) receives live
 * pipeline results via Phoenix channels. This bridge connects the Node.js
 * GitLab connector to the ABZU IDE's WebSocket.
 *
 * ABZU channel: "sovereign:gitlab"
 * Events pushed:
 *   "pipeline_result"  — Frankenstein 4-step result with WORM seals
 *   "gitlab_event"     — raw classified event (push/MR/pipeline)
 *   "worm_seal"        — new chain entry notification
 *
 * The ABZU IDE renders:
 *   - Live Brain output (architect spec)
 *   - Live Legs code (Granite execution)
 *   - WORM chain visualisation
 *   - GitLab event feed
 */

import { createHash } from 'crypto'

const ABZU_URL    = process.env.ABZU_URL    || 'http://localhost:4000'
const ABZU_SECRET = process.env.ABZU_SECRET || 'sovereign-abzu-bridge'

// Phoenix channel HTTP push (via Phoenix.PubSub REST bridge on ABZU side)
// ABZU exposes POST /api/pipeline_event for server-to-server push
export async function pushToAbzu(payload) {
  const ts   = Date.now()
  const sig  = createHash('sha256')
    .update(`${ABZU_SECRET}:${ts}:${JSON.stringify(payload)}`)
    .digest('hex')
    .slice(0, 16)

  const body = {
    channel:   'sovereign:gitlab',
    event:     'pipeline_result',
    payload,
    ts,
    sig,
    worm_hash: payload.worm,
    seal:      '⬡ Ω ↺ Ψ Δ Λ Σ Φ α',
  }

  try {
    const res = await fetch(`${ABZU_URL}/api/pipeline_event`, {
      method:  'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Sovereign-Sig': sig,
        'X-Sovereign-Ts':  ts.toString(),
      },
      body:   JSON.stringify(body),
      signal: AbortSignal.timeout(5_000),
    })

    if (res.ok) {
      console.log(`[ABZU] Pipeline result pushed → ${ABZU_URL} ✓`)
    } else {
      console.warn(`[ABZU] Push failed: HTTP ${res.status}`)
    }
  } catch (err) {
    // ABZU IDE offline — log but don't fail the pipeline
    console.warn(`[ABZU] IDE offline (${err.message}) — result logged to WORM only`)
  }
}

// ── Format for ABZU LiveView rendering ───────────────────────────────────────
export function formatForAbzu(frankensteinResult, gitlabContext) {
  const steps = frankensteinResult.steps || []
  return {
    pipeline_id: Date.now(),
    event:       gitlabContext,
    brain: {
      output: steps.find(s => s.step === 'BRAIN')?.output || '',
      ms:     steps.find(s => s.step === 'BRAIN')?.ms || 0,
      worm:   steps.find(s => s.step === 'BRAIN')?.worm || '',
    },
    hands: {
      output: steps.find(s => s.step === 'HANDS')?.output || '',
      ms:     steps.find(s => s.step === 'HANDS')?.ms || 0,
    },
    legs: {
      output: steps.find(s => s.step === 'LEGS')?.output || '',
      ms:     steps.find(s => s.step === 'LEGS')?.ms || 0,
      worm:   steps.find(s => s.step === 'LEGS')?.worm || '',
      model:  steps.find(s => s.step === 'LEGS')?.model || '',
    },
    review: {
      output: frankensteinResult.final_out || '',
      worm:   steps.find(s => s.step === 'REVIEW')?.worm || '',
    },
    chain_hash: frankensteinResult.worm?.hash || '',
    total_seals: frankensteinResult.worm?.seals || 0,
  }
}
