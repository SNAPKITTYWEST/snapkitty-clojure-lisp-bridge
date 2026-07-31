import { createHash }  from 'crypto'
import { appendFile, readFile, mkdir } from 'fs/promises'
import { existsSync } from 'fs'
import { join } from 'path'

const WORM_DIR  = join(process.cwd(), '.worm')
const WORM_FILE = join(WORM_DIR, 'gitlab_chain.jsonl')

let prevHash = '0'.repeat(64)

async function ensureDir() {
  if (!existsSync(WORM_DIR)) await mkdir(WORM_DIR, { recursive: true })
}

export async function wormSeal(data) {
  await ensureDir()
  const ts        = Date.now()
  const this_hash = createHash('sha256')
    .update(`${prevHash}:${ts}:${JSON.stringify(data)}`)
    .digest('hex')

  const entry = { ts, prev_hash: prevHash, this_hash, ...data }
  await appendFile(WORM_FILE, JSON.stringify(entry) + '\n')
  prevHash = this_hash
  return entry
}

export async function readWorm(limit = 20) {
  await ensureDir()
  try {
    const lines = (await readFile(WORM_FILE, 'utf8'))
      .split('\n').filter(Boolean).map(l => JSON.parse(l))
    return lines.slice(-limit).reverse()
  } catch { return [] }
}
