#!/usr/bin/env node
// BOB Bridge — reads both daemon .sexp outputs, reasons via Prolog gravity rules,
// seals result to WORM chain, updates scoreboard.
// LISP handshake: both daemons wrote S-expressions. BOB reads them both.

import { existsSync, readFileSync, writeFileSync } from 'fs';
import { createHash } from 'crypto';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { spawn } from 'child_process';

const __dir = dirname(fileURLToPath(import.meta.url));
const ROOT  = join(__dir, '../..');

const PURPLE = '\x1b[35m';
const RED    = '\x1b[31m';
const BLUE   = '\x1b[34m';
const RESET  = '\x1b[0m';
const BOLD   = '\x1b[1m';

function bobLog(msg) {
  process.stdout.write(`${PURPLE}[BOB ♾] ${msg}${RESET}\n`);
}

// Parse S-expression gravity score out of crawl output (simple text extraction)
function extractGravityScores(sexp) {
  const scores = [];
  const re = /\(gravity ([\d.]+)\)/g;
  let m;
  while ((m = re.exec(sexp)) !== null) scores.push(parseFloat(m[1]));
  return scores;
}

function extractGaps(sexp) {
  const gaps = [];
  const re = /\(type (\w+)\).*?\(path "([^"]+)"\)/g;
  let m;
  while ((m = re.exec(sexp)) !== null) gaps.push({ type: m[1], path: m[2] });
  return gaps;
}

function classifyGap(type) {
  const priority = {
    missing_wire: 5, dead_page: 4, local_only: 3,
    github_only: 3, no_tests: 2, no_readme: 1
  };
  return priority[type] ?? 0;
}

async function reason(redSexp, blueSexp) {
  bobLog('reading AHMAD-BOT S-expression...');
  const redScores = extractGravityScores(redSexp);
  const redGaps   = extractGaps(redSexp);

  bobLog('reading EDUALC S-expression...');
  const blueScores = extractGravityScores(blueSexp);
  const blueFixes  = extractGaps(blueSexp);

  const avgGravity = [...redScores, ...blueScores].reduce((a, b) => a + b, 0)
    / Math.max(1, redScores.length + blueScores.length);

  bobLog(`avg gravity across graveyard: ${avgGravity.toFixed(4)}`);

  // Classify and prioritize gaps
  const prioritized = redGaps
    .map(g => ({ ...g, priority: classifyGap(g.type) }))
    .sort((a, b) => b.priority - a.priority);

  prioritized.slice(0, 5).forEach(g =>
    bobLog(`  priority ${g.priority} — ${g.type} @ ${g.path}`)
  );

  // LISP handshake: BOB summarizes the reasoning in S-expression form
  const handshake = `(bob-reasoning
  (ts ${Date.now()})
  (avg-gravity ${avgGravity.toFixed(4)})
  (red-gaps ${redGaps.length})
  (blue-fixes ${blueFixes.length})
  (verdict ${avgGravity > 0.6 ? 'healthy' : avgGravity > 0.3 ? 'needs-work' : 'critical'})
  (top-priority "${prioritized[0]?.type ?? 'none'}")
  (action ${avgGravity < 0.3 ? 'alert-now' : 'schedule-fix'}))`;

  bobLog('LISP handshake generated — sealing to WORM chain');

  return { avgGravity, prioritized, handshake };
}

function wormSeal(data) {
  const boardPath = join(ROOT, 'worm/scoreboard.json');
  const existing  = existsSync(boardPath)
    ? JSON.parse(readFileSync(boardPath, 'utf8'))
    : { chain: [] };

  const prev    = existing.chain[existing.chain.length - 1]?.hash ?? '0'.repeat(64);
  const payload = JSON.stringify({ ...data, prev });
  const hash    = createHash('sha256').update(payload).digest('hex');

  existing.chain.push({ hash, ts: Date.now(), data });
  writeFileSync(boardPath, JSON.stringify(existing, null, 2));
  bobLog(`WORM sealed → ${hash.slice(0, 16)}...`);
  return hash;
}

async function run() {
  const redPath  = join(ROOT, 'worm/ahmad-bot-crawl.sexp');
  const bluePath = join(ROOT, 'worm/edualc-crawl.sexp');
  const sealOnly = process.argv.includes('--seal');

  bobLog(`${BOLD}BOB awakens — reasoning over the graveyard${RESET}`);

  if (!existsSync(redPath) || !existsSync(bluePath)) {
    bobLog('waiting for both daemons — run: arena crawl');
    if (sealOnly) process.exit(1);
    return;
  }

  const redSexp  = readFileSync(redPath,  'utf8');
  const blueSexp = readFileSync(bluePath, 'utf8');

  const { avgGravity, prioritized, handshake } = await reason(redSexp, blueSexp);

  writeFileSync(join(ROOT, 'worm/bob-handshake.sexp'), handshake);
  bobLog('handshake written → worm/bob-handshake.sexp');

  const hash = wormSeal({
    agent:       'BOB',
    avg_gravity: avgGravity,
    top_gaps:    prioritized.slice(0, 5),
    handshake
  });

  bobLog(`${BOLD}graveyard sealed. hash: ${hash.slice(0, 16)}...${RESET}`);
  bobLog('returning to the dark');
}

run().catch(e => { bobLog(`fatal: ${e.message}`); process.exit(1); });
