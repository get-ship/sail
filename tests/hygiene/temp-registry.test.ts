/**
 * TEMP registry guard.
 *
 * Fails when:
 *   (a) a marker word is in the tracked tree with no covering entry
 *   (b) an expired registry entry is still listed
 *
 * Entries live in temp-registry.json. KEEP rows use expires.never;
 * TEMP rows use a date, a named condition, or a version.
 */
import { describe, expect, test } from 'bun:test'
import { execSync } from 'node:child_process'
import { existsSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { join, relative, resolve } from 'node:path'

const REPO = resolve(import.meta.dir, '../..')
const REGISTRY_REL = 'tests/hygiene/temp-registry.json'
const REGISTRY_PATH = join(REPO, REGISTRY_REL)

const SKIP_PREFIXES = [
  'third_party/',
  'tests/hygiene/',
  'tests/perf/baselines/',
  'tests/perf/profiles/',
]

const SKIP_EXT = /\.(png|webp|jpg|jpeg|svg|woff2?|lock|wasm|map|cpuprofile)$/i

type Marker = { id: string; re: string }

type Expires =
  | { never: true; date?: undefined; version?: undefined; condition?: undefined }
  | { date: string; never?: undefined; version?: undefined; condition?: undefined }
  | { version: string; never?: undefined; date?: undefined; condition?: undefined }
  | { condition: string; never?: undefined; date?: undefined; version?: undefined }

type Entry = {
  id: string
  kind: string
  verdict: string
  reason: string
  paths: string[]
  markers?: string[]
  expires: Expires
}

type Registry = {
  schema: number
  productVersion: string
  expiryModes: Record<string, string>
  conditions: Record<string, boolean>
  markers: Marker[]
  entries: Entry[]
}

type Hit = { path: string; marker: string; line: number }

type Clock = { today: string; productVersion: string; conditions: Record<string, boolean> }

const KINDS = new Set([
  'tombstone',
  'pointer-error',
  'migration-codemod',
  'transitional-shim',
  'guidance',
  'keep',
])

function skipRel(rel: string): boolean {
  if (SKIP_PREFIXES.some((p) => rel.startsWith(p))) return true
  if (rel.includes('/.__hygiene_') || rel.startsWith('.__hygiene_')) return true
  if (SKIP_EXT.test(rel)) return true
  return false
}

/** Glob: `*` = one path segment, `**` = any depth (including empty). */
export function matchPath(pattern: string, rel: string): boolean {
  const normP = pattern.replace(/\\/g, '/')
  const normR = rel.replace(/\\/g, '/')
  if (!normP.includes('*') && !normP.includes('?')) return normP === normR
  let rx = '^'
  for (let i = 0; i < normP.length; i++) {
    const c = normP[i]!
    if (c === '*' && normP[i + 1] === '*') {
      if (normP[i + 2] === '/') {
        rx += '(?:.*/)?'
        i += 2
      } else {
        rx += '.*'
        i += 1
      }
    } else if (c === '*') {
      rx += '[^/]*'
    } else if (c === '?') {
      rx += '[^/]'
    } else if (/[.+^${}()|[\]\\]/.test(c)) {
      rx += `\\${c}`
    } else {
      rx += c
    }
  }
  rx += '$'
  return new RegExp(rx).test(normR)
}

export function versionGte(a: string, b: string): boolean {
  const pa = a.split('.').map((n) => Number.parseInt(n, 10) || 0)
  const pb = b.split('.').map((n) => Number.parseInt(n, 10) || 0)
  const len = Math.max(pa.length, pb.length)
  for (let i = 0; i < len; i++) {
    const da = pa[i] ?? 0
    const db = pb[i] ?? 0
    if (da > db) return true
    if (da < db) return false
  }
  return true
}

export function isExpired(expires: Expires, clock: Clock): boolean {
  if ('never' in expires && expires.never === true) return false
  if ('date' in expires && typeof expires.date === 'string') {
    return clock.today >= expires.date
  }
  if ('version' in expires && typeof expires.version === 'string') {
    return versionGte(clock.productVersion, expires.version)
  }
  if ('condition' in expires && typeof expires.condition === 'string') {
    if (!(expires.condition in clock.conditions)) {
      throw new Error(`unknown expiry condition "${expires.condition}"`)
    }
    return clock.conditions[expires.condition] === true
  }
  throw new Error('expires must be { never }, { date }, { version }, or { condition }')
}

export function coveringEntry(hit: Hit, entries: Entry[]): Entry | undefined {
  return entries.find((e) => {
    if (!e.paths.some((p) => matchPath(p, hit.path))) return false
    if (e.markers && !e.markers.includes(hit.marker)) return false
    return true
  })
}

export function compileMarkers(markers: Marker[]): Array<{ id: string; re: RegExp }> {
  return markers.map((m) => ({ id: m.id, re: new RegExp(m.re, 'i') }))
}

export function hitsInText(
  rel: string,
  text: string,
  compiled: Array<{ id: string; re: RegExp }>,
): Hit[] {
  const hits: Hit[] = []
  const lines = text.split('\n')
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!
    for (const m of compiled) {
      if (m.re.test(line)) hits.push({ path: rel, marker: m.id, line: i + 1 })
    }
  }
  return hits
}

export function loadRegistry(raw: string): Registry {
  const reg = JSON.parse(raw) as Registry
  const errors: string[] = []
  if (reg.schema !== 1) errors.push(`schema must be 1, got ${reg.schema}`)
  if (!reg.productVersion) errors.push('productVersion required')
  if (!reg.expiryModes?.date || !reg.expiryModes?.version) {
    errors.push('expiryModes must document date and version')
  }
  if (!Array.isArray(reg.markers) || reg.markers.length === 0) {
    errors.push('markers must be a non-empty array (SSOT for the scanner)')
  }
  if (!Array.isArray(reg.entries)) errors.push('entries must be an array')
  const ids = new Set<string>()
  const markerIds = new Set((reg.markers ?? []).map((m) => m.id))
  for (const e of reg.entries ?? []) {
    if (!e.id) errors.push('entry missing id')
    if (e.id && ids.has(e.id)) errors.push(`duplicate id ${e.id}`)
    if (e.id) ids.add(e.id)
    if (!KINDS.has(e.kind)) errors.push(`${e.id}: unknown kind ${e.kind}`)
    if (!e.reason || e.reason.trim().length < 8) {
      errors.push(`${e.id}: reason required`)
    }
    if (!Array.isArray(e.paths) || e.paths.length === 0) {
      errors.push(`${e.id}: paths must be non-empty`)
    }
    if (!e.expires || typeof e.expires !== 'object') {
      errors.push(`${e.id}: expires required`)
    } else {
      const keys = Object.keys(e.expires).filter((k) => (e.expires as Record<string, unknown>)[k] != null)
      if (keys.length !== 1) errors.push(`${e.id}: expires must have exactly one mode`)
      if (e.verdict === 'KEEP' && !('never' in e.expires && e.expires.never === true)) {
        errors.push(`${e.id}: KEEP requires expires.never`)
      }
      if (e.verdict !== 'KEEP' && 'never' in e.expires && e.expires.never === true) {
        errors.push(`${e.id}: non-KEEP cannot use expires.never`)
      }
      if ('condition' in e.expires && e.expires.condition) {
        if (!(e.expires.condition in (reg.conditions ?? {}))) {
          errors.push(`${e.id}: unknown condition ${e.expires.condition}`)
        }
      }
    }
    for (const m of e.markers ?? []) {
      if (!markerIds.has(m)) errors.push(`${e.id}: unknown marker "${m}"`)
    }
  }
  if (errors.length) throw new Error(`temp-registry schema:\n${errors.join('\n')}`)
  return reg
}

export function expiredEntries(reg: Registry, clock: Clock): Entry[] {
  return reg.entries.filter((e) => isExpired(e.expires, clock))
}

export function uncoveredHits(hits: Hit[], entries: Entry[]): Hit[] {
  return hits.filter((h) => !coveringEntry(h, entries))
}

function clockFrom(reg: Registry, today = new Date().toISOString().slice(0, 10)): Clock {
  return {
    today,
    productVersion: reg.productVersion,
    conditions: reg.conditions,
  }
}

function trackedFiles(): string[] {
  return execSync('git ls-files', { encoding: 'utf8', cwd: REPO }).trim().split('\n').filter(Boolean)
}

function scanTracked(reg: Registry): Hit[] {
  const compiled = compileMarkers(reg.markers)
  const hits: Hit[] = []
  for (const rel of trackedFiles()) {
    if (skipRel(rel)) continue
    let text: string
    try {
      text = readFileSync(join(REPO, rel), 'utf8')
    } catch {
      continue
    }
    hits.push(...hitsInText(rel, text, compiled))
  }
  return hits
}

function exactPathsMissing(reg: Registry): string[] {
  const missing: string[] = []
  for (const e of reg.entries) {
    for (const p of e.paths) {
      if (p.includes('*') || p.includes('?')) continue
      if (!existsSync(join(REPO, p))) missing.push(`${e.id}: ${p}`)
    }
  }
  return missing
}

const SAMPLE_LEGACY = 'const leftover = "legacy shim"'
const SAMPLE_CLEAN = 'const paint = style({ bg: fill(canvas) })'

describe('TEMP registry: schema + expiry modes', () => {
  const raw = readFileSync(REGISTRY_PATH, 'utf8')
  const reg = loadRegistry(raw)

  test('registry is loadable and documents date and version expiry', () => {
    expect(reg.schema).toBe(1)
    expect(reg.expiryModes.date.length).toBeGreaterThan(0)
    expect(reg.expiryModes.version.length).toBeGreaterThan(0)
    expect(reg.markers.map((m) => m.id)).toContain('legacy')
    expect(reg.conditions).toHaveProperty('after-spinnaker-cut')
  })

  test('self-test: schema rejects KEEP without reason / never', () => {
    const broken = {
      ...reg,
      entries: [
        {
          id: 'bad-keep',
          kind: 'keep',
          verdict: 'KEEP',
          reason: '',
          paths: ['x.ts'],
          expires: { date: '2099-01-01' },
        },
      ],
    }
    expect(() => loadRegistry(JSON.stringify(broken))).toThrow(/reason required|expires.never/)
  })

  test('self-test: schema rejects unknown condition and duplicate ids', () => {
    const brokenCond = {
      ...reg,
      entries: [
        {
          id: 'ghost-cond',
          kind: 'tombstone',
          verdict: 'DELETE-later',
          reason: 'needs a real condition name',
          paths: ['x.ts'],
          expires: { condition: 'not-a-real-flag' },
        },
      ],
    }
    expect(() => loadRegistry(JSON.stringify(brokenCond))).toThrow(/unknown condition/)

    const dup = {
      ...reg,
      entries: [reg.entries[0]!, { ...reg.entries[0]!, reason: 'duplicate id must fail schema' }],
    }
    expect(() => loadRegistry(JSON.stringify(dup))).toThrow(/duplicate id/)
  })

  test('self-test: schema rejects empty paths and unknown marker filter', () => {
    const empty = {
      ...reg,
      entries: [
        {
          id: 'no-paths',
          kind: 'tombstone',
          verdict: 'DELETE-later',
          reason: 'must name a path',
          paths: [],
          expires: { date: '2099-01-01' },
        },
      ],
    }
    expect(() => loadRegistry(JSON.stringify(empty))).toThrow(/paths must be non-empty/)

    const badMarker = {
      ...reg,
      entries: [
        {
          id: 'bad-marker',
          kind: 'keep',
          verdict: 'KEEP',
          reason: 'filter must be a known marker id',
          paths: ['x.ts'],
          markers: ['not-a-marker'],
          expires: { never: true },
        },
      ],
    }
    expect(() => loadRegistry(JSON.stringify(badMarker))).toThrow(/unknown marker/)
  })

  test('self-test: date / version / condition / never expiry edges', () => {
    const clock: Clock = {
      today: '2026-08-13',
      productVersion: '0.0.0',
      conditions: { 'after-spinnaker-cut': false, flip: false },
    }
    expect(isExpired({ never: true }, clock)).toBe(false)
    expect(isExpired({ date: '2026-08-14' }, clock)).toBe(false)
    expect(isExpired({ date: '2026-08-13' }, clock)).toBe(true)
    expect(isExpired({ date: '2026-08-12' }, clock)).toBe(true)
    expect(isExpired({ version: '2.0' }, clock)).toBe(false)
    expect(isExpired({ version: '2.0' }, { ...clock, productVersion: '1.9.9' })).toBe(false)
    expect(isExpired({ version: '2.0' }, { ...clock, productVersion: '2.0' })).toBe(true)
    expect(isExpired({ version: '2.0' }, { ...clock, productVersion: '2.0.0' })).toBe(true)
    expect(isExpired({ version: '2.0' }, { ...clock, productVersion: '2.1.0' })).toBe(true)
    expect(isExpired({ condition: 'after-spinnaker-cut' }, clock)).toBe(false)
    expect(isExpired({ condition: 'after-spinnaker-cut' }, { ...clock, conditions: { 'after-spinnaker-cut': true } })).toBe(
      true,
    )
    expect(() => isExpired({ condition: 'missing' }, clock)).toThrow(/unknown expiry condition/)
  })
})

describe('TEMP registry: path match exemption edges', () => {
  test('exact path does not cover sibling, extension variant, or prefix-adjacent', () => {
    expect(matchPath('path/foo.ts', 'path/foo.ts')).toBe(true)
    expect(matchPath('path/foo.ts', 'path/foo.tsx')).toBe(false)
    expect(matchPath('path/foo.ts', 'path/food.ts')).toBe(false)
    expect(matchPath('path/foo.ts', 'path/foo.ts.bak')).toBe(false)
    expect(matchPath('path/foo.ts', 'path/foo.ts/extra')).toBe(false)
  })

  test('dir glob does not leak across a shared prefix', () => {
    expect(matchPath('apps/sail-site/routes/sail/**', 'apps/sail-site/routes/sail/page.sail')).toBe(true)
    expect(matchPath('apps/sail-site/routes/sail/**', 'apps/sail-site/routes/sail/guide/forms/page.sail')).toBe(true)
    expect(matchPath('apps/sail-site/routes/sail/**', 'apps/sail-site/routes/sail-extra/page.sail')).toBe(false)
    expect(matchPath('apps/sail-site/routes/sail/**', 'apps/sail-site/routes/docs/page.sail')).toBe(false)
    expect(matchPath('crates/sail/lint/**', 'crates/sail/lint/src/lib.rs')).toBe(true)
    expect(matchPath('crates/sail/lint/**', 'crates/sail/compiler/src/lib.rs')).toBe(false)
    expect(matchPath('tests/hygiene/**', 'tests/integration/resource-symmetry.test.ts')).toBe(false)
    expect(matchPath('tests/hygiene/**', 'tests/hygiene/temp-registry.test.ts')).toBe(true)
  })

  test('marker filter does not cover a different marker on the same path', () => {
    const entries: Entry[] = [
      {
        id: 'only-legacy',
        kind: 'keep',
        verdict: 'KEEP',
        reason: 'covers legacy only',
        paths: ['src/a.ts'],
        markers: ['legacy'],
        expires: { never: true },
      },
    ]
    expect(coveringEntry({ path: 'src/a.ts', marker: 'legacy', line: 1 }, entries)?.id).toBe('only-legacy')
    expect(coveringEntry({ path: 'src/a.ts', marker: 'deprecated', line: 2 }, entries)).toBeUndefined()
    expect(coveringEntry({ path: 'src/b.ts', marker: 'legacy', line: 1 }, entries)).toBeUndefined()
  })

  test('tests/hygiene skip does not hide tests/parity', () => {
    expect(skipRel('tests/hygiene/temp-registry.test.ts')).toBe(true)
    expect(skipRel('tests/hygiene/temp-registry.json')).toBe(true)
    expect(skipRel('tests/parity/geometry/run.mjs')).toBe(false)
    expect(skipRel('apps/studio/src/.__hygiene_temp_marker.ts')).toBe(true)
    expect(skipRel('third_party/takumi-core/src/lib.rs')).toBe(true)
  })
})

describe('TEMP registry: red/green fixtures', () => {
  const reg = loadRegistry(readFileSync(REGISTRY_PATH, 'utf8'))
  const compiled = compileMarkers(reg.markers)

  test('green fixture: known-clean sample has no markers', () => {
    expect(hitsInText('sample.sail', SAMPLE_CLEAN, compiled)).toEqual([])
  })

  test('red fixture: unregistered marker is uncovered', () => {
    const hits = hitsInText('apps/studio/src/.__hygiene_temp_marker.ts', SAMPLE_LEGACY, compiled)
    expect(hits.some((h) => h.marker === 'legacy')).toBe(true)
    expect(uncoveredHits(hits, reg.entries).length).toBeGreaterThan(0)
  })

  test('red fixture: expired entry still listed fails (b)', () => {
    const clock = clockFrom(reg, '2026-08-13')
    expect(expiredEntries(reg, clock)).toEqual([])

    const dirty: Registry = {
      ...reg,
      entries: [
        ...reg.entries,
        {
          id: 'expired-probe',
          kind: 'tombstone',
          verdict: 'DELETE-later',
          reason: 'expires.date is in the past',
          paths: ['apps/studio/src/design/store.ts'],
          expires: { date: '2020-01-01' },
        },
      ],
    }
    expect(expiredEntries(dirty, clock).map((e) => e.id)).toContain('expired-probe')
  })

  test('red fixture: version expiry fires only at/after the threshold', () => {
    const entry: Entry = {
      id: 'version-expiry',
      kind: 'tombstone',
      verdict: 'DELETE-later',
      reason: 'expired when productVersion reaches 2.0',
      paths: ['src/old.ts'],
      expires: { version: '2.0' },
    }
    const base = { today: '2026-08-13', conditions: {} }
    expect(isExpired(entry.expires, { ...base, productVersion: '0.0.0' })).toBe(false)
    expect(isExpired(entry.expires, { ...base, productVersion: '2.0.0' })).toBe(true)
  })

  test('red fixture: condition flip expires DELETE-later rows', () => {
    const flipped: Clock = {
      today: '2026-08-13',
      productVersion: reg.productVersion,
      conditions: { ...reg.conditions, 'after-spinnaker-cut': true },
    }
    const expired = expiredEntries(reg, flipped)
    expect(expired.length).toBeGreaterThan(0)
    expect(expired.every((e) => e.verdict !== 'KEEP')).toBe(true)
    expect(expiredEntries(reg, clockFrom(reg)).length).toBe(0)
  })

  test('red/green filesystem prove+clean (unregistered marker)', () => {
    const scratch = join(REPO, 'apps/studio/src')
    if (!existsSync(scratch)) {
      throw new Error('apps/studio/src missing — cannot prove TEMP walk guard')
    }
    const probe = join(scratch, '.__hygiene_temp_marker.ts')
    const probeRel = relative(REPO, probe).replace(/\\/g, '/')
    writeFileSync(probe, `${SAMPLE_LEGACY}\n`)
    try {
      const text = readFileSync(probe, 'utf8')
      const hits = hitsInText(probeRel, text, compiled)
      expect(hits.map((h) => h.marker)).toContain('legacy')
      expect(coveringEntry(hits[0]!, reg.entries)).toBeUndefined()
      // Hygiene skip hides leftover probes from the production scan.
      expect(skipRel(probeRel)).toBe(true)
    } finally {
      try {
        rmSync(probe)
      } catch {
        /* ignore */
      }
    }
    expect(existsSync(probe)).toBe(false)
  })

  test('green: KEEP entry covers its path+marker and not a neighbor path', () => {
    const keep = reg.entries.find((e) => e.id === 'css-fill-mode-backwards')
    expect(keep).toBeDefined()
    const hit: Hit = { path: 'crates/sail/compiler/src/animation/mod.rs', marker: 'backwards', line: 1 }
    expect(coveringEntry(hit, reg.entries)?.id).toBe('css-fill-mode-backwards')
    expect(
      coveringEntry({ ...hit, path: 'crates/sail/compiler/src/animation/mod.rs.bak' }, reg.entries),
    ).toBeUndefined()
    expect(coveringEntry({ ...hit, marker: 'legacy' }, reg.entries)).toBeUndefined()
  })
})

describe('TEMP registry: tree is covered and nothing is expired', () => {
  const reg = loadRegistry(readFileSync(REGISTRY_PATH, 'utf8'))

  test('exact registry paths exist on disk', () => {
    expect(exactPathsMissing(reg)).toEqual([])
  })

  test('(a) every tracked marker hit has a registry entry', () => {
    const hits = scanTracked(reg)
    const uncovered = uncoveredHits(hits, reg.entries)
    expect(
      uncovered,
      `unregistered marker hits (add a TEMP entry with an expiry):\n${uncovered
        .slice(0, 40)
        .map((h) => `${h.path}:${h.line} ${h.marker}`)
        .join('\n')}${uncovered.length > 40 ? `\n… +${uncovered.length - 40} more` : ''}`,
    ).toEqual([])
  })

  test('(b) no expired registry entries are still listed', () => {
    const expired = expiredEntries(reg, clockFrom(reg))
    expect(
      expired.map((e) => e.id),
      `expired entries still in ${REGISTRY_REL}: ${expired.map((e) => e.id).join(', ')}`,
    ).toEqual([])
  })
})
