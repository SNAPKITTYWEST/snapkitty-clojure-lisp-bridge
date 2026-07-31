/**
 * SNAPKITTY GITLAB — Webhook Receiver
 * Receives GitLab events → routes to ROBOB shadow orchestrator → ABZU IDE live push
 *
 * Handles:
 *   Push events       — code change → Brain/Hands/Legs analysis
 *   MR events         — merge request → sovereign code review
 *   Pipeline events   — CI failure → auto-diagnose via Frankenstein
 *   Note events       — /snapkitty comment in MR → trigger workflow
 *   Job events        — job failure → Legs proposes fix
 *
 * ⬡ Ω ↺ Ψ Δ Λ Σ Φ α
 */

import Fastify from 'fastify'
import cors    from '@fastify/cors'
import crypto  from 'crypto'
import { route } from '../orchestrator/robob.mjs'
import { pushToAbzu } from '../abzu/bridge.mjs'
import { wormSeal } from '../worm/chain.mjs'

const app = Fastify({ logger: { level: 'info' } })
await app.register(cors, { origin: '*' })

const GITLAB_SECRET = process.env.GITLAB_WEBHOOK_SECRET || 'sovereign-snapkitty'
const PORT          = parseInt(process.env.PORT || '4700')

// ── Verify GitLab webhook signature ──────────────────────────────────────────
function verifyToken(req) {
  const token = req.headers['x-gitlab-token']
  if (!token) return false
  return crypto.timingSafeEqual(
    Buffer.from(token),
    Buffer.from(GITLAB_SECRET)
  )
}

// ── Event classifier → ROBOB pipeline selector ───────────────────────────────
function classifyEvent(event, body) {
  switch (event) {
    case 'Push Hook':
      return {
        pipeline: 'code-review',
        context: {
          repo:     body.project?.name,
          branch:   body.ref?.replace('refs/heads/', ''),
          commits:  body.commits?.slice(0, 5).map(c => ({ id: c.id?.slice(0,8), msg: c.message })),
          author:   body.user_name,
          diff_url: body.project?.web_url,
        }
      }

    case 'Merge Request Hook':
      return {
        pipeline: 'mr-review',
        context: {
          mr_id:      body.object_attributes?.iid,
          title:      body.object_attributes?.title,
          source:     body.object_attributes?.source_branch,
          target:     body.object_attributes?.target_branch,
          action:     body.object_attributes?.action,
          url:        body.object_attributes?.url,
          diff:       body.object_attributes?.description,
          author:     body.user?.username,
        }
      }

    case 'Pipeline Hook':
      return {
        pipeline: 'pipeline-diagnose',
        context: {
          pipeline_id: body.object_attributes?.id,
          status:      body.object_attributes?.status,
          ref:         body.object_attributes?.ref,
          failed_jobs: body.builds?.filter(b => b.status === 'failed').map(b => ({
            name: b.name, stage: b.stage
          })),
          project:     body.project?.name,
          url:         body.project?.web_url + '/-/pipelines/' + body.object_attributes?.id,
        }
      }

    case 'Note Hook':
    case 'Merge Request Note Hook': {
      const note = body.object_attributes?.note || ''
      if (!note.startsWith('/snapkitty') && !note.startsWith('/sk ')) return null
      const command = note.replace(/^\/(snapkitty|sk)\s*/, '').trim()
      return {
        pipeline: 'direct-command',
        context: {
          command,
          mr_id:    body.merge_request?.iid,
          note_id:  body.object_attributes?.id,
          author:   body.user?.username,
          url:      body.object_attributes?.url,
        }
      }
    }

    case 'Job Hook':
      if (body.build_status !== 'failed') return null
      return {
        pipeline: 'job-fix',
        context: {
          job_id:    body.build_id,
          job_name:  body.build_name,
          stage:     body.build_stage,
          log_url:   body.repository?.homepage + '/-/jobs/' + body.build_id,
          project:   body.project_name,
          ref:       body.ref,
        }
      }

    default:
      return null
  }
}

// ── POST /webhook — main GitLab event receiver ────────────────────────────────
app.post('/webhook', async (req, reply) => {
  if (!verifyToken(req)) {
    return reply.code(401).send({ error: 'Invalid webhook token' })
  }

  const event = req.headers['x-gitlab-event']
  const body  = req.body

  console.log(`[GITLAB] ${event} — ${body.project?.name || 'unknown'}`)

  const classified = classifyEvent(event, body)
  if (!classified) {
    return reply.send({ status: 'ignored', event })
  }

  // WORM seal the incoming event
  const seal = await wormSeal({
    event:    'GITLAB_WEBHOOK',
    pipeline: classified.pipeline,
    context:  classified.context,
  })

  // Fire ROBOB shadow orchestrator async — don't block webhook response
  setImmediate(async () => {
    try {
      const result = await route(classified.pipeline, classified.context, seal.this_hash)

      // Push result to ABZU IDE live view
      await pushToAbzu({
        event:    event,
        pipeline: classified.pipeline,
        context:  classified.context,
        result,
        worm:     seal.this_hash,
      })
    } catch (err) {
      console.error(`[ROBOB ERROR] ${err.message}`)
    }
  })

  reply.send({
    status:   'accepted',
    pipeline: classified.pipeline,
    worm:     seal.this_hash,
    seal:     '⬡ Ω ↺ Ψ Δ Λ Σ Φ α',
  })
})

// ── GET /health ───────────────────────────────────────────────────────────────
app.get('/health', async () => ({
  status:     'SNAPKITTY_GITLAB_ONLINE',
  pipelines:  ['code-review','mr-review','pipeline-diagnose','direct-command','job-fix'],
  orchestrator: 'ROBOB shadow',
  abzu_ide:   process.env.ABZU_URL || 'ws://localhost:4000/socket',
  worm:       'sealed',
}))

await app.listen({ port: PORT, host: '0.0.0.0' })
console.log(`\n⬡ SNAPKITTY GITLAB CONNECTOR — port ${PORT}`)
console.log(`  Webhook:     POST /webhook  (X-Gitlab-Token: ${GITLAB_SECRET})`)
console.log(`  Pipelines:   code-review · mr-review · diagnose · command · job-fix`)
console.log(`  Orchestrator: ROBOB shadow`)
console.log(`  ABZU IDE:    ${process.env.ABZU_URL || 'ws://localhost:4000/socket'}\n`)
