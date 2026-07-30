#!/usr/bin/env node
// EDUALC — blue hat crawler (Claude inverted)
// Finds what can be fixed, wired, and restored.
// Reports to BOB via NATS arena.crawl.blue

import { Octokit } from '@octokit/rest';
import { connect } from 'nats';
import { createHash } from 'crypto';
import { writeFileSync } from 'fs';

const ORG = 'SNAPKITTYWEST';
const BLUE  = '\x1b[34m';
const RESET = '\x1b[0m';
const BOLD  = '\x1b[1m';

function blueLog(msg) {
  process.stdout.write(`${BLUE}[EDUALC 🔵] ${msg}${RESET}\n`);
}

async function proposeReadme(repo) {
  return `# ${repo.name}\n\nSovereign agent module — part of the SnapKitty graveyard restoration.\n`;
}

async function restoreRepo(octokit, repo) {
  const fixes = [];

  try {
    const { data: contents } = await octokit.repos.getContent({
      owner: ORG, repo: repo.name, path: ''
    });

    const hasReadme = contents.some(f => f.name.toLowerCase().startsWith('readme'));
    const hasTests  = contents.some(f =>
      ['test', 'tests', '__tests__', 'spec'].includes(f.name.toLowerCase())
    );

    if (!hasReadme) {
      const draft = await proposeReadme(repo);
      fixes.push({
        type:    'no_readme',
        path:    `${repo.name}/README.md`,
        action:  'create',
        content: draft
      });
      blueLog(`  🔧 proposed README → ${repo.name}`);
    }

    if (!hasTests && repo.size > 0) {
      fixes.push({
        type:   'no_tests',
        path:   `${repo.name}/test/`,
        action: 'scaffold',
        content: '// test scaffold — EDUALC proposes, human approves'
      });
      blueLog(`  🔧 proposed test scaffold → ${repo.name}`);
    }

    if (repo.size === 0) {
      fixes.push({
        type:   'dead_page',
        path:   repo.name,
        action: 'flag-for-review',
        content: 'empty repo — archive or populate'
      });
      blueLog(`  📌 flagged empty → ${repo.name}`);
    }

  } catch {
    fixes.push({
      type:   'missing_wire',
      path:   repo.name,
      action: 'investigate',
      content: 'cannot read repo — permissions or deletion needed'
    });
    blueLog(`  📌 flagged inaccessible → ${repo.name}`);
  }

  return { name: repo.name, url: repo.html_url, fixes };
}

async function run() {
  const token = process.env.GITHUB_TOKEN;
  if (!token) { blueLog('GITHUB_TOKEN not set'); process.exit(1); }

  const octokit = new Octokit({ auth: token });

  blueLog(`${BOLD}entering graveyard — restoration mode — org: ${ORG}${RESET}`);

  let repos;
  try {
    ({ data: repos } = await octokit.repos.listForOrg({ org: ORG, per_page: 100 }));
  } catch {
    try {
      ({ data: repos } = await octokit.repos.listForUser({ username: ORG, per_page: 100 }));
    } catch {
      const { execSync } = await import('child_process');
      repos = JSON.parse(execSync(`gh repo list ${ORG} --limit 100 --json name,url,description,defaultBranchRef,diskUsage`, { encoding: 'utf8' }));
      repos = repos.map(r => ({ name: r.name, html_url: r.url, size: r.diskUsage ?? 0 }));
    }
  }
  blueLog(`found ${repos.length} repos — scanning for what can be restored`);

  const results = [];
  for (const repo of repos) {
    blueLog(`restoring → ${repo.name}`);
    const result = await restoreRepo(octokit, repo);
    blueLog(`  fixes proposed: ${result.fixes.length}`);
    results.push(result);
  }

  const totalFixes = results.reduce((n, r) => n + r.fixes.length, 0);
  blueLog(`restoration scan complete — ${results.length} repos | ${totalFixes} fixes proposed`);

  const sexp = toSexp(results);
  const hash = createHash('sha256').update(sexp).digest('hex');

  writeFileSync('worm/edualc-crawl.sexp', sexp);
  blueLog(`sealed → worm/edualc-crawl.sexp (${hash.slice(0, 12)}...)`);

  if (process.env.NATS_URL) {
    try {
      const nc = await connect({ servers: process.env.NATS_URL, timeout: 3000 });
      const payload = JSON.stringify({ agent: 'EDUALC', hash, results });
      await nc.publish('arena.crawl.blue', new TextEncoder().encode(payload));
      await nc.drain();
      blueLog('published → arena.crawl.blue');
    } catch {
      blueLog('NATS unavailable — result written to file only');
    }
  } else {
    blueLog('NATS_URL not set — file only mode');
  }

  blueLog('crawling back into the dark');
}

function toSexp(results) {
  const nodes = results.map(r => {
    const fixes = r.fixes.map(f =>
      `(fix (type ${f.type}) (path "${f.path}") (action ${f.action}))`
    ).join('\n    ');
    return `  (repo
    (name "${r.name}")
    (url "${r.url}")
    (fixes
    ${fixes || '(none)'}))`;
  }).join('\n');

  return `(edualc-crawl
  (agent "EDUALC")
  (hat blue)
  (ts ${Date.now()})
${nodes})`;
}

run().catch(e => { blueLog(`fatal: ${e.message}`); process.exit(1); });
