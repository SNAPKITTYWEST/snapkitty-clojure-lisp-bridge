import { Octokit } from '@octokit/rest';

const octokit = new Octokit({ auth: process.env.GITHUB_TOKEN });

const forkOwner = 'SNAPKITTYWEST';
const forkRepo  = 'multiplicity';
const targetOwner = 'MultiplicityTheory';
const targetRepo  = 'multiplicity';
const branch = 'sovereign-worm-injection';

const projectLisp = `(defproject MultiplicityTheory/multiplicity
  :sovereign-entry "lean/AffineCore/MTPI/PIRTM.lean"
  :constitution "Lean 4 axiom-clean core — MOC operator calculus"
  :gravity-score 0.60
  :gaps [:open-admits :reversed-stability-condition :unregistered-morphisms]
  :restoration-path
    [(fix-admits "lean/AffineCore/MTPI/PIRTM.lean")
     (resolve-stability-contradiction :alpha>=1 :vs :q<1)
     (register-morphisms-in-RegHom)
     (seal-to-worm-chain)]
  :worm-hook "SNAPKITTYWEST/agentic-arena"
  :joint-seal "994afb3b8a036d6f"
  :crawled-by [:AHMAD-BOT :EDUALC]
  :oracle :BOB)`;

const forthMap = `: gravity 0.60 ;
: alive? gravity 0.6 >= ;
: constitutional-gap? s" admit" has-token? ;
: stability-contradiction? s" alpha>=1" has-token? s" q<1" has-token? xor ;
: run-inversion
  cr ." [WORM] MultiplicityTheory/multiplicity"
  cr ." gravity: " gravity .
  cr ." gaps: open-admits, reversed-stability-condition"
  cr ." crawled by AGENTIC ARENA — SNAPKITTYWEST"
  constitutional-gap? if cr ." CONSTITUTIONAL GAP DETECTED" then
  stability-contradiction? if cr ." STABILITY CONTRADICTION OPEN" then
  seal ;
run-inversion`;

const hookJson = JSON.stringify({
  worm: 'SNAPKITTYWEST/agentic-arena',
  joint_seal: '994afb3b8a036d6f',
  gravity: 0.60,
  gaps: ['open-admits', 'reversed-stability-condition'],
  crawled_at: new Date().toISOString(),
  oracle: 'BOB',
  message: 'Restoration proposed. Accept = join the chain. Reject = no harm done.'
}, null, 2);

const prBody = `## Graveyard Restoration Proposal

The AGENTIC ARENA benevolent worm has crawled this repository.

**Gravity score:** 0.60 — alive but constitutional gaps detected.

**Gaps identified:**
- Open \`admit\` placeholders in \`lean/AffineCore/MTPI/PIRTM.lean\` — theorems declared but unproven
- Reversed stability condition: spec states α ≥ 1 but contractive dynamics requires q < 1 (open contradiction)
- Morphism registry (RegHom) not formally instantiated

**What this PR contains:**
- \`Project.lisp\` — sovereign build manifest with restoration path
- \`graveyard.forth\` — Forth map of the repo topology and gap analysis
- \`.sovereign/worm-hook.json\` — WORM chain registration

**To accept:** merge this PR and join the restoration chain. Your repo gets sealed to the joint SHA-256 proof: \`994afb3b8a036d6f\`

**To reject:** close the PR. No harm done. The crawl record remains in the chain.

---
*Crawled by AHMAD-BOT [red] + EDUALC [blue] | Oracle: BOB | Chain: SNAPKITTYWEST/agentic-arena*`;

async function run() {
  // Get fork base SHA
  let baseSha;
  try {
    const { data: ref } = await octokit.git.getRef({ owner: forkOwner, repo: forkRepo, ref: 'heads/master' });
    baseSha = ref.object.sha;
  } catch {
    const { data: ref } = await octokit.git.getRef({ owner: forkOwner, repo: forkRepo, ref: 'heads/main' });
    baseSha = ref.object.sha;
  }
  console.log('[WORM] base sha:', baseSha.slice(0, 8));

  const { data: baseCommit } = await octokit.git.getCommit({ owner: forkOwner, repo: forkRepo, commit_sha: baseSha });

  const [b1, b2, b3] = await Promise.all([
    octokit.git.createBlob({ owner: forkOwner, repo: forkRepo, content: projectLisp, encoding: 'utf-8' }),
    octokit.git.createBlob({ owner: forkOwner, repo: forkRepo, content: forthMap,    encoding: 'utf-8' }),
    octokit.git.createBlob({ owner: forkOwner, repo: forkRepo, content: hookJson,    encoding: 'utf-8' })
  ]);

  const { data: tree } = await octokit.git.createTree({
    owner: forkOwner, repo: forkRepo,
    base_tree: baseCommit.tree.sha,
    tree: [
      { path: 'Project.lisp',              mode: '100644', type: 'blob', sha: b1.data.sha },
      { path: 'graveyard.forth',           mode: '100644', type: 'blob', sha: b2.data.sha },
      { path: '.sovereign/worm-hook.json', mode: '100644', type: 'blob', sha: b3.data.sha }
    ]
  });

  const { data: commit } = await octokit.git.createCommit({
    owner: forkOwner, repo: forkRepo,
    message: 'sovereign: worm injection — AGENTIC ARENA graveyard map',
    tree: tree.sha,
    parents: [baseSha]
  });

  // Push branch to fork
  try {
    await octokit.git.createRef({ owner: forkOwner, repo: forkRepo, ref: `refs/heads/${branch}`, sha: commit.sha });
  } catch {
    await octokit.git.updateRef({ owner: forkOwner, repo: forkRepo, ref: `heads/${branch}`, sha: commit.sha, force: true });
  }
  console.log('[WORM] branch pushed to fork:', branch);

  // Open PR from fork → original
  const { data: pr } = await octokit.pulls.create({
    owner: targetOwner,
    repo:  targetRepo,
    title: 'sovereign: graveyard restoration proposal — AGENTIC ARENA',
    head:  `${forkOwner}:${branch}`,
    base:  'main',
    body:  prBody
  });

  console.log('[WORM] PR opened:', pr.html_url);
  console.log('[WORM] sealed to chain — crawl complete');
}

run().catch(err => {
  console.error('[WORM] error:', err.message);
  process.exit(1);
});
