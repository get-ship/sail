#!/usr/bin/env node
/**
 * Tiny CLI for packages/sail/l1-audit/matrix/*.json
 *
 *   node bin/l1-matrix.mjs list
 *   node bin/l1-matrix.mjs get button
 *   node bin/l1-matrix.mjs summary
 *   node bin/l1-matrix.mjs set button gates.operator.status=approved
 *   node bin/l1-matrix.mjs seed          # create stub JSON for stdlib tags missing a matrix row
 */
import { readdirSync, readFileSync, writeFileSync, existsSync, mkdirSync } from 'node:fs'
import { dirname, join, basename } from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))
const ROOT = join(HERE, '..')
const MATRIX = join(ROOT, 'matrix')
const STDLIB = join(ROOT, '../../../crates/sail/compiler/stdlib')

const cmd = process.argv[2] ?? 'summary'
const args = process.argv.slice(3)

function loadAll() {
  if (!existsSync(MATRIX)) return []
  return readdirSync(MATRIX)
    .filter((f) => f.endsWith('.json'))
    .map((f) => JSON.parse(readFileSync(join(MATRIX, f), 'utf8')))
    .sort((a, b) => a.tag.localeCompare(b.tag))
}

function load(tag) {
  const p = join(MATRIX, `${tag}.json`)
  if (!existsSync(p)) {
    console.error(`no matrix row: ${tag}`)
    process.exit(1)
  }
  return { path: p, data: JSON.parse(readFileSync(p, 'utf8')) }
}

function setPath(obj, dotted, value) {
  const parts = dotted.split('.')
  let cur = obj
  for (let i = 0; i < parts.length - 1; i++) {
    const k = parts[i]
    if (cur[k] == null || typeof cur[k] !== 'object') cur[k] = {}
    cur = cur[k]
  }
  const last = parts[parts.length - 1]
  let v = value
  if (value === 'true') v = true
  else if (value === 'false') v = false
  else if (value === 'null') v = null
  cur[last] = v
}

function stub(tag) {
  return {
    tag,
    status: 'todo',
    stdlib: `crates/sail/compiler/stdlib/${tag}.sail`,
    tokens: { component: [], notes: '' },
    props: {},
    patterns: [],
    gates: {
      variants: { status: 'todo' },
      themes: { status: 'todo' },
      drift: { status: 'todo' },
      detail: { status: 'todo' },
      operator: { status: 'pending' },
    },
    gaps: [],
    updated: new Date().toISOString().slice(0, 10),
  }
}

if (cmd === 'list') {
  for (const row of loadAll()) {
    console.log(`${row.tag}\t${row.status}\top=${row.gates?.operator?.status ?? '?'}`)
  }
} else if (cmd === 'get') {
  const tag = args[0]
  if (!tag) {
    console.error('usage: get <tag>')
    process.exit(2)
  }
  console.log(JSON.stringify(load(tag).data, null, 2))
} else if (cmd === 'summary') {
  const rows = loadAll()
  const by = { todo: 0, in_progress: 0, approved: 0, blocked: 0, other: 0 }
  for (const r of rows) {
    const s = r.status in by ? r.status : 'other'
    by[s]++
  }
  console.log(`rows ${rows.length}`)
  console.log(JSON.stringify(by, null, 2))
  const pending = rows.filter((r) => r.gates?.operator?.status !== 'approved')
  if (pending.length) {
    console.log('operator pending:')
    for (const r of pending) console.log(`  - ${r.tag} (${r.status})`)
  }
} else if (cmd === 'set') {
  // set button gates.operator.status=approved
  const tag = args[0]
  const assign = args[1]
  if (!tag || !assign || !assign.includes('=')) {
    console.error('usage: set <tag> path.to.field=value')
    process.exit(2)
  }
  const [path, ...rest] = assign.split('=')
  const value = rest.join('=')
  const { path: file, data } = load(tag)
  setPath(data, path, value)
  data.updated = new Date().toISOString().slice(0, 10)
  writeFileSync(file, JSON.stringify(data, null, 2) + '\n')
  console.log(`updated ${tag} ${path}=${value}`)
} else if (cmd === 'seed') {
  mkdirSync(MATRIX, { recursive: true })
  if (!existsSync(STDLIB)) {
    console.error(`stdlib not found at ${STDLIB}`)
    process.exit(1)
  }
  const tags = readdirSync(STDLIB)
    .filter((f) => f.endsWith('.sail') && f !== 'index.sail')
    .map((f) => basename(f, '.sail'))
  let n = 0
  for (const tag of tags) {
    const p = join(MATRIX, `${tag}.json`)
    if (existsSync(p)) continue
    writeFileSync(p, JSON.stringify(stub(tag), null, 2) + '\n')
    n++
  }
  console.log(`seeded ${n} new rows (${tags.length} stdlib tags)`)
} else {
  console.error(`unknown command: ${cmd}`)
  console.error('commands: list | get | summary | set | seed')
  process.exit(2)
}
