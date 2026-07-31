/**
 * ROBOB Shadow Orchestrator — GitLab Pipeline Router
 *
 * ROBOB receives the classified GitLab event, selects the correct
 * Frankenstein workflow pipeline, fires Brain → Hands → Legs → Review,
 * then posts the result back to GitLab via API and to ABZU IDE via WebSocket.
 *
 * Architecture:
 *   GitLab event
 *     → classifyEvent()            [webhook/server.mjs]
 *     → ROBOB.route()              [this file]
 *         → buildPrompt()          [prompt per pipeline type]
 *         → frankenstein()         [POST /workflow to Frankenstein :4300]
 *         → postToGitLab()         [MR comment / status / pipeline note]
 *     → pushToAbzu()               [abzu/bridge.mjs — WebSocket push]
 */

import { postToGitLab } from '../gitlab/api.mjs'

const FRANKENSTEIN_URL = process.env.FRANKENSTEIN_URL || 'http://localhost:4300'

// ── Pipeline prompt builders ──────────────────────────────────────────────────
const PIPELINE_PROMPTS = {

  'code-review': (ctx) => `
You are performing a sovereign code review for a GitLab push event.

Repository: ${ctx.repo}
Branch: ${ctx.branch}
Author: ${ctx.author}
Recent commits:
${ctx.commits?.map(c => `  [${c.id}] ${c.msg}`).join('\n') || 'none'}

Perform a sovereign code review. Check for:
1. Security vulnerabilities (injection, auth bypass, secrets in code)
2. Trust Deed violations (mutable audit trails, bypassed governance)
3. Double-entry violations if accounting code is present
4. Architecture regressions from WORM/sovereign patterns
5. Missing error boundaries on async operations

Output: structured review with PASS/WARN/BLOCK verdict per category.
`.trim(),

  'mr-review': (ctx) => `
You are performing a sovereign merge request review.

MR #${ctx.mr_id}: "${ctx.title}"
${ctx.source} → ${ctx.target}
Author: ${ctx.author}
Action: ${ctx.action}
Description: ${ctx.diff || 'none'}

Sovereign MR review:
1. Security gate — injection, auth, secrets
2. Trust Deed compliance — governance, WORM chain integrity
3. Breaking changes to any public API surface
4. Missing tests for critical paths
5. Overall APPROVE / REQUEST_CHANGES / BLOCK verdict

Format as a GitLab MR comment with emoji status indicators.
`.trim(),

  'pipeline-diagnose': (ctx) => `
You are diagnosing a failed GitLab CI pipeline.

Project: ${ctx.project}
Pipeline #${ctx.pipeline_id} on ${ctx.ref}
Status: ${ctx.status}
Failed jobs:
${ctx.failed_jobs?.map(j => `  - ${j.stage}/${j.name}`).join('\n') || 'none listed'}

Pipeline URL: ${ctx.url}

Diagnose the failure and provide:
1. Most likely root cause (be specific — don't say "check the logs")
2. Exact fix — commands or code changes
3. Prevention — what to add to catch this earlier
4. Confidence level (HIGH/MEDIUM/LOW) with reasoning

Output actionable fix that the developer can apply in under 5 minutes.
`.trim(),

  'direct-command': (ctx) => `
A developer issued a SnapKitty command in a GitLab MR comment.

Command: ${ctx.command}
Author: ${ctx.author}
MR: #${ctx.mr_id || 'N/A'}

Execute this sovereign command and return the result.
Format as a clean GitLab comment reply.
`.trim(),

  'job-fix': (ctx) => `
A GitLab CI job failed and needs a sovereign fix proposal.

Project: ${ctx.project}
Job: ${ctx.job_name} (stage: ${ctx.stage})
Branch: ${ctx.ref}
Log URL: ${ctx.log_url}

Based on the job name and stage, propose:
1. The most likely failure cause
2. Exact .gitlab-ci.yml fix or code fix
3. Shell commands to reproduce and verify locally

Output a concrete fix, ready to copy-paste.
`.trim(),
}

// ── Call Frankenstein ─────────────────────────────────────────────────────────
async function frankenstein(prompt) {
  const res = await fetch(`${FRANKENSTEIN_URL}/workflow`, {
    method:  'POST',
    headers: { 'Content-Type': 'application/json' },
    body:    JSON.stringify({ prompt }),
    signal:  AbortSignal.timeout(180_000), // 3 min max
  })
  if (!res.ok) throw new Error(`Frankenstein HTTP ${res.status}`)
  return res.json()
}

// ── ROBOB route — main orchestrator entry point ───────────────────────────────
export async function route(pipeline, context, wormHash) {
  const promptFn = PIPELINE_PROMPTS[pipeline]
  if (!promptFn) throw new Error(`Unknown pipeline: ${pipeline}`)

  const prompt = promptFn(context)
  console.log(`[ROBOB] Pipeline: ${pipeline} | WORM: ${wormHash}`)

  // Fire Frankenstein (Brain → Hands → Legs → Review)
  const result = await frankenstein(prompt)

  // Post result back to GitLab
  await postToGitLab(pipeline, context, result).catch(err =>
    console.warn(`[GITLAB API] Post failed: ${err.message}`)
  )

  return result
}
