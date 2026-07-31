/**
 * GitLab API — post results back as MR comments, pipeline notes, status
 */

const GITLAB_URL   = process.env.GITLAB_URL   || 'https://gitlab.com'
const GITLAB_TOKEN = process.env.GITLAB_TOKEN || ''

async function glFetch(path, opts = {}) {
  const res = await fetch(`${GITLAB_URL}/api/v4${path}`, {
    ...opts,
    headers: {
      'Content-Type': 'application/json',
      'PRIVATE-TOKEN': GITLAB_TOKEN,
      ...(opts.headers || {}),
    },
  })
  return res
}

// ── Post MR comment ───────────────────────────────────────────────────────────
async function postMRComment(projectId, mrIid, body) {
  if (!GITLAB_TOKEN || !projectId) return
  await glFetch(`/projects/${encodeURIComponent(projectId)}/merge_requests/${mrIid}/notes`, {
    method: 'POST',
    body:   JSON.stringify({ body }),
  })
}

// ── Post commit comment ───────────────────────────────────────────────────────
async function postCommitComment(projectId, sha, note) {
  if (!GITLAB_TOKEN || !projectId || !sha) return
  await glFetch(`/projects/${encodeURIComponent(projectId)}/repository/commits/${sha}/comments`, {
    method: 'POST',
    body:   JSON.stringify({ note }),
  })
}

// ── Format Frankenstein result as GitLab markdown comment ─────────────────────
function formatGitLabComment(pipeline, result) {
  const steps  = result.steps || []
  const review = result.final_out || ''
  const worm   = result.worm?.hash || ''

  const brain  = steps.find(s => s.step === 'BRAIN')
  const legs   = steps.find(s => s.step === 'LEGS')

  const verdict = review.match(/PASS|APPROVE/i)          ? '✅'
    : review.match(/WARN|REQUEST_CHANGES/i)               ? '⚠️'
    : review.match(/BLOCK|REJECT/i)                       ? '🚫'
    : '🔍'

  return `## ${verdict} SnapKitty Sovereign Review

> Pipeline: \`${pipeline}\` · WORM: \`${worm}\` · ⬡ Ω ↺ Ψ Δ Λ Σ Φ α

### 🧠 Brain Analysis
${brain?.output?.slice(0, 800) || '_Brain offline_'}

### 🦵 Legs Implementation
\`\`\`
${legs?.output?.slice(0, 600) || '_Legs offline_'}
\`\`\`

### 🔍 Sovereign Review
${review?.slice(0, 1000) || '_No review_'}

---
*Powered by [SnapKitty](https://snapkitty.dev) · Frankenstein workflow · Granite Code 3B local GPU*
`
}

// ── Route result back to correct GitLab endpoint ──────────────────────────────
export async function postToGitLab(pipeline, context, result) {
  if (!GITLAB_TOKEN) {
    console.log('[GITLAB API] No token — skipping post-back')
    return
  }

  const comment = formatGitLabComment(pipeline, result)

  switch (pipeline) {
    case 'mr-review':
    case 'direct-command':
      if (context.mr_id) {
        const projectId = process.env.GITLAB_PROJECT_ID || context.project_id
        await postMRComment(projectId, context.mr_id, comment)
        console.log(`[GITLAB API] Comment posted to MR #${context.mr_id}`)
      }
      break

    case 'code-review':
      // Post to latest commit
      console.log(`[GITLAB API] Code review complete for ${context.repo}@${context.branch}`)
      // Would post to commit: await postCommitComment(projectId, latestSha, comment)
      break

    case 'pipeline-diagnose':
    case 'job-fix':
      // In production: post to pipeline notes or create issue
      console.log(`[GITLAB API] Pipeline diagnosis ready for #${context.pipeline_id}`)
      break
  }
}
