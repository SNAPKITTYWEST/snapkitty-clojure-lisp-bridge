#!/usr/bin/env node
// THE RANSOM-WORM INJECTOR
// Opens a PR on a target repo proposing sovereign structure.
// NOT malware. PRs are opt-in. The repo owner accepts or rejects.
// The "ransom" is the offer: accept sovereign structure, or stay chaotic.
// The "infection" is the PR. The "recovery" is the inverted meta-repo.

import { Octokit } from '@octokit/rest';
import { renderOrgAsForth } from '../forth/render.mjs';
import { createHash } from 'crypto';
import { writeFileSync, mkdirSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dir = dirname(fileURLToPath(import.meta.url));
const ROOT  = join(__dir, '../..');

const RED    = '\x1b[31m';
const PURPLE = '\x1b[35m';
const GREEN  = '\x1b[32m';
const RESET  = '\x1b[0m';
const BOLD   = '\x1b[1m';

function log(msg, color = PURPLE) {
  process.stdout.write(`${color}[INJECTOR] ${msg}${RESET}\n`);
}

// ── Crawl target repo gravity ─────────────────────────────
async function crawlTarget(octokit, owner, repo) {
  log(`crawling target: ${owner}/${repo}`);
  const result = { name: repo, owner, gravity: 0, status: 'orphan', clusters: [], gaps: [] };

  try {
    const { data: contents } = await octokit.repos.getContent({ owner, repo, path: '' });

    const hasReadme  = contents.some(f => f.name.toLowerCase().startsWith('readme'));
    const hasTests   = contents.some(f => ['test','tests','__tests__','spec'].includes(f.name.toLowerCase()));
    const hasBuild   = contents.some(f => ['makefile','package.json','cargo.toml','build.gradle'].includes(f.name.toLowerCase()));
    const hasCI      = contents.some(f => f.name === '.github');
    const hasDocs    = contents.some(f => ['docs','doc','documentation'].includes(f.name.toLowerCase()));

    // Build clusters from root structure
    for (const item of contents) {
      if (item.type === 'dir') {
        result.clusters.push({
          id:      item.name,
          gravity: 0.5, // default — deep crawl would refine this
          status:  'unknown',
          members: []
        });
      }
    }

    // Score gravity
    const score = (
      (hasReadme  ? 0.2 : 0) +
      (hasTests   ? 0.2 : 0) +
      (hasBuild   ? 0.2 : 0) +
      (hasCI      ? 0.2 : 0) +
      (hasDocs    ? 0.2 : 0)
    );
    result.gravity = score;
    result.status  = score >= 0.6 ? 'alive' : score >= 0.3 ? 'broken' : 'orphan';

    // Identify gaps
    if (!hasReadme)  result.gaps.push({ type: 'no_readme',  path: 'README.md', fix: 'add sovereign README', gravity: 0.1 });
    if (!hasTests)   result.gaps.push({ type: 'no_tests',   path: 'tests/',    fix: 'add test suite',      gravity: 0.2 });
    if (!hasBuild)   result.gaps.push({ type: 'missing_wire', path: 'build',   fix: 'add Project.lisp',    gravity: 0.3 });
    if (!hasDocs)    result.gaps.push({ type: 'dead_page',  path: 'docs/',     fix: 'add docs',            gravity: 0.1 });

    log(`target gravity: ${score.toFixed(2)} | status: ${result.status} | gaps: ${result.gaps.length}`);

  } catch (e) {
    log(`cannot read target: ${e.message}`, RED);
    result.gaps.push({ type: 'missing_wire', path: repo, fix: 'check permissions', gravity: 0 });
  }

  return result;
}

// ── Generate Project.lisp payload ────────────────────────
function generateProjectLisp(target) {
  return `;;; Project.lisp — Sovereign Build Manifest
;;; Injected by AGENTIC-ARENA ransom-worm
;;; Gravity: ${target.gravity.toFixed(4)} | Status: ${target.status}
;;; This file replaces your legacy build system.
;;; Feed it to the Sovereign LISP Machine.

(defproject ${target.name}
  :gravity  ${target.gravity.toFixed(4)}
  :status   :${target.status}
  :sovereign t

  :build-steps
  '((clean   . "remove build artifacts")
    (test    . "run sovereign test suite")
    (verify  . "run Lean 4 proofs")
    (seal    . "WORM-seal the build")
    (deploy  . "deploy to GitHub Pages"))

  :gaps
  '(${target.gaps.map(g => `(:${g.type} "${g.path}" "${g.fix}")`).join('\n    ')})

  :worm-hook
  '(:endpoint "https://github.com/SNAPKITTYWEST/agentic-arena"
    :seal-on  :build-complete
    :chain    :bifrost))
`;
}

// ── Generate the Forth graveyard map ─────────────────────
function generateForthMap(target) {
  return renderOrgAsForth(
    `${target.owner}/${target.name}`,
    [{ name: target.name, gravity: target.gravity, status: target.status }]
  );
}

// ── Open the PR ───────────────────────────────────────────
async function openPR(octokit, target, projectLisp, forthMap) {
  const { owner, name: repo } = target;
  const branch = 'sovereign-inversion';

  log(`injecting into ${owner}/${repo} via PR...`);

  try {
    // Get default branch SHA
    const { data: repoData } = await octokit.repos.get({ owner, repo });
    const defaultBranch = repoData.default_branch;
    const { data: ref } = await octokit.git.getRef({ owner, repo, ref: `heads/${defaultBranch}` });
    const baseSha = ref.object.sha;

    // Create branch
    await octokit.git.createRef({
      owner, repo,
      ref: `refs/heads/${branch}`,
      sha: baseSha
    }).catch(() => log('branch may already exist — continuing'));

    // Create tree with new files
    const { data: tree } = await octokit.git.createTree({
      owner, repo,
      base_tree: baseSha,
      tree: [
        {
          path: 'Project.lisp',
          mode: '100644',
          type: 'blob',
          content: projectLisp
        },
        {
          path: 'graveyard.forth',
          mode: '100644',
          type: 'blob',
          content: forthMap
        },
        {
          path: '.sovereign/worm-hook.json',
          mode: '100644',
          type: 'blob',
          content: JSON.stringify({
            injected_by: 'SNAPKITTYWEST/agentic-arena',
            gravity:     target.gravity,
            status:      target.status,
            ts:          Date.now(),
            worm_seal:   createHash('sha256').update(projectLisp).digest('hex').slice(0, 16)
          }, null, 2)
        }
      ]
    });

    // Create commit
    const { data: commit } = await octokit.git.createCommit({
      owner, repo,
      message: `feat: sovereign inversion — Project.lisp + graveyard.forth\n\nInjected by AGENTIC-ARENA ransom-worm.\nGravity: ${target.gravity.toFixed(4)} | Status: ${target.status}\n\nThis PR replaces legacy build scripts with sovereign structure.\nAccept it to join the graveyard restoration. Reject it to stay chaotic.`,
      tree:    tree.sha,
      parents: [baseSha]
    });

    // Update branch
    await octokit.git.updateRef({
      owner, repo,
      ref:   `heads/${branch}`,
      sha:   commit.sha,
      force: true
    });

    // Open PR
    const { data: pr } = await octokit.pulls.create({
      owner, repo,
      title: `[AGENTIC-ARENA] Sovereign Inversion — gravity: ${target.gravity.toFixed(2)}`,
      head:  branch,
      base:  defaultBranch,
      body: `## Sovereign Inversion Proposal

**Gravity score:** \`${target.gravity.toFixed(4)}\` — this repo is **${target.status}**

The ransom-worm has crawled this repository and found **${target.gaps.length} gaps**:

${target.gaps.map(g => `- \`${g.type}\` @ \`${g.path}\` → ${g.fix}`).join('\n')}

### What this PR adds

| File | Purpose |
|------|---------|
| \`Project.lisp\` | Sovereign build manifest — replaces legacy build scripts |
| \`graveyard.forth\` | Forth-rendered architecture map — ancient, executable documentation |
| \`.sovereign/worm-hook.json\` | WORM chain hook — seals every build to the bifrost chain |

### The deal

Accept this PR → your repo joins the graveyard restoration. Every build is sealed, every gap is tracked, every change is immutable and verifiable.

Reject this PR → no harm done. The worm crawls back into the dark.

---
*Generated by [AGENTIC-ARENA](https://github.com/SNAPKITTYWEST/agentic-arena) — the benevolent graveyard crawler.*`
    });

    log(`PR opened: ${pr.html_url}`, GREEN);
    return pr;

  } catch (e) {
    log(`PR failed: ${e.message}`, RED);
    return null;
  }
}

// ── MAIN ─────────────────────────────────────────────────
async function run() {
  const token  = process.env.GITHUB_TOKEN;
  const target = process.argv[2]; // e.g. "owner/repo"

  if (!token) { log('GITHUB_TOKEN not set'); process.exit(1); }
  if (!target || !target.includes('/')) {
    log('usage: node injector/index.mjs owner/repo');
    process.exit(1);
  }

  const [owner, repo] = target.split('/');
  const octokit = new Octokit({ auth: token });

  log(`${BOLD}ransom-worm awakens — target: ${owner}/${repo}${RESET}`);

  const crawlResult = await crawlTarget(octokit, owner, repo);
  const projectLisp = generateProjectLisp(crawlResult);
  const forthMap    = generateForthMap(crawlResult);

  // Write meta-repo locally
  const outDir = join(ROOT, 'worm/meta-repos', `${owner}-${repo}`);
  if (!existsSync(outDir)) mkdirSync(outDir, { recursive: true });
  writeFileSync(join(outDir, 'Project.lisp'),     projectLisp);
  writeFileSync(join(outDir, 'graveyard.forth'),  forthMap);
  log(`meta-repo written → worm/meta-repos/${owner}-${repo}/`);

  // Open the PR
  await openPR(octokit, crawlResult, projectLisp, forthMap);

  log(`${BOLD}worm complete — crawling back into the dark${RESET}`);
}

run().catch(e => { log(`fatal: ${e.message}`, RED); process.exit(1); });
