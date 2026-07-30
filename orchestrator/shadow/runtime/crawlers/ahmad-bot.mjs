#!/usr/bin/env node
// AHMAD-BOT — red hat crawler
// Finds broken pipes, dead pages, disconnected repos in the graveyard.
// Reports to BOB via NATS arena.crawl.red

import { Octokit } from '@octokit/rest';
import { connect } from 'nats';
import { createHash } from 'crypto';
import { writeFileSync } from 'fs';

const ORG = 'SNAPKITTYWEST';
const RED = '\x1b[31m';
const RESET = '\x1b[0m';
const BOLD = '\x1b[1m';

function redLog(msg) {
  process.stdout.write(`${RED}[AHMAD-BOT 🔴] ${msg}${RESET}\n`);
}

function gravity(lines, depFanOut, recursionDepth, totalLines) {
  if (totalLines === 0) return 0;
  return (lines * depFanOut * recursionDepth) / totalLines;
}

async function crawlRepo(octokit, repo) {
  const gaps = [];
  let score = 0;

  try {
    const { data: contents } = await octokit.repos.getContent({
      owner: ORG, repo: repo.name, path: ''
    });

    const hasReadme  = contents.some(f => f.name.toLowerCase().startsWith('readme'));
    const hasTests   = contents.some(f =>
      ['test', 'tests', '__tests__', 'spec'].includes(f.name.toLowerCase())
    );
    const hasPackage = contents.some(f => f.name === 'package.json');
    const hasCargo   = contents.some(f => f.name === 'Cargo.toml');
    const hasAction  = contents.some(f => f.name === '.github');

    if (!hasReadme) {
      gaps.push({ type: 'no_readme',  path: repo.name, gravity: 0.1, fix: 'EDUALC writes README' });
      redLog(`  ☠ no README — ${repo.name}`);
    }
    if (!hasTests) {
      gaps.push({ type: 'no_tests',   path: repo.name, gravity: 0.2, fix: 'EDUALC writes first test' });
      redLog(`  ☠ no tests — ${repo.name}`);
    }

    const depCount   = (hasPackage || hasCargo) ? 1 : 0;
    const depth      = hasAction ? 2 : 1;
    score = gravity(repo.size, depCount + 1, depth, 1000);

    if (repo.size === 0) {
      gaps.push({ type: 'dead_page', path: repo.name, gravity: 0, fix: 'mark grave or delete' });
      redLog(`  💀 empty repo — ${repo.name}`);
    }

  } catch {
    gaps.push({ type: 'missing_wire', path: repo.name, gravity: 0, fix: 'check permissions or delete' });
    redLog(`  ⚡ cannot read — ${repo.name}`);
  }

  return {
    name:    repo.name,
    url:     repo.html_url,
    gravity: Math.min(score, 1.0).toFixed(4),
    status:  score >= 0.6 ? 'alive' : score >= 0.3 ? 'broken' : 'orphan',
    gaps
  };
}

async function run() {
  const token = process.env.GITHUB_TOKEN;
  if (!token) { redLog('GITHUB_TOKEN not set'); process.exit(1); }

  const octokit = new Octokit({ auth: token });

  redLog(`${BOLD}entering graveyard — org: ${ORG}${RESET}`);

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
  redLog(`found ${repos.length} repos — beginning gravity crawl`);

  const results = [];
  for (const repo of repos) {
    redLog(`scanning → ${repo.name}`);
    const result = await crawlRepo(octokit, repo);
    redLog(`  gravity: ${result.gravity} | status: ${result.status} | gaps: ${result.gaps.length}`);
    results.push(result);
  }

  const totalGaps = results.reduce((n, r) => n + r.gaps.length, 0);
  redLog(`crawl complete — ${results.length} repos | ${totalGaps} gaps found`);

  const sexp = toSexp(results);
  const hash = createHash('sha256').update(sexp).digest('hex');

  writeFileSync('worm/ahmad-bot-crawl.sexp', sexp);
  redLog(`sealed → worm/ahmad-bot-crawl.sexp (${hash.slice(0, 12)}...)`);

  // publish to NATS if available — skip on Windows if NATS not running
  if (process.env.NATS_URL) {
    try {
      const nc = await connect({ servers: process.env.NATS_URL, timeout: 3000 });
      const payload = JSON.stringify({ agent: 'AHMAD-BOT', hash, results });
      await nc.publish('arena.crawl.red', new TextEncoder().encode(payload));
      await nc.drain();
      redLog('published → arena.crawl.red');
    } catch {
      redLog('NATS unavailable — result written to file only');
    }
  } else {
    redLog('NATS_URL not set — file only mode');
  }

  redLog('crawling back into the dark');
}

function toSexp(results) {
  const nodes = results.map(r => {
    const gaps = r.gaps.map(g =>
      `(gap (type ${g.type}) (path "${g.path}") (gravity ${g.gravity}) (fix "${g.fix}"))`
    ).join('\n    ');
    return `  (repo
    (name "${r.name}")
    (url "${r.url}")
    (gravity ${r.gravity})
    (status ${r.status})
    (gaps
    ${gaps || '(none)'}))`;
  }).join('\n');

  return `(ahmad-bot-crawl
  (agent "AHMAD-BOT")
  (hat red)
  (ts ${Date.now()})
${nodes})`;
}

run().catch(e => { redLog(`fatal: ${e.message}`); process.exit(1); });
