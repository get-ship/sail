// Build script: read the @ship/sail-tutorial lessons and emit a single typed
// data module the docs routes import. We DO NOT rewrite the tutorial — we
// load `packages/sail/tutorial/src/prose/<slug>.md` (flat files: frontmatter +
// body) and the paired `lessons/<slug>/{starter,solved}/` sources (one `.sail`
// per dir, stem = custom-element tag, controller block inline when the lesson
// has state), render the prose body to HTML here, and write
// `src/compositions/tutorial-data.ts` (import-convention home).
//
// Run: `node scripts/gen-tutorial.mjs` (from apps/sail-site). The output is
// committed so the deck build needs no extra step.
import { readFileSync, writeFileSync, existsSync, readdirSync } from 'node:fs'
import { join, dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const REPO = resolve(__dirname, '../../..')
const TUT = join(REPO, 'packages/sail/tutorial/src')
const PROSE = join(TUT, 'prose')
const LESSONS = join(TUT, 'lessons')
const OUT = resolve(__dirname, '../src/compositions/tutorial-data.ts')

// ── tiny markdown → HTML ────────────────────────────────────────────────────
// Handles: ATX headings, paragraphs, ul/ol lists, blockquotes, inline code,
// bold, italic, links, and ```fenced``` blocks → <pre class="code">.
function escapeHtml(s) {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

function inline(s) {
  let out = escapeHtml(s)
  // inline code first so its contents aren't re-processed
  out = out.replace(/`([^`]+)`/g, (_m, c) => `<code>${c}</code>`)
  out = out.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
  out = out.replace(/(^|[^*])\*([^*\n]+)\*/g, '$1<em>$2</em>')
  out = out.replace(
    /\[([^\]]+)\]\(([^)\s]+)\)/g,
    (_m, t, h) => `<a href="${h}" rel="noopener noreferrer">${t}</a>`,
  )
  return out
}

function renderMarkdown(src) {
  const lines = src.replace(/\r\n/g, '\n').split('\n')
  const cur = () => lines[i] ?? ''
  const out = []
  let i = 0
  while (i < lines.length) {
    const line = cur()
    if (/^```/.test(line)) {
      const buf = []
      i++
      while (i < lines.length && !/^```/.test(cur())) {
        buf.push(cur())
        i++
      }
      i++ // closing fence
      out.push(`<pre class="code"><code>${escapeHtml(buf.join('\n'))}</code></pre>`)
      continue
    }
    const h = /^(#{1,6})\s+(.*)$/.exec(line)
    if (h) {
      const n = h[1].length
      out.push(`<h${n}>${inline(h[2])}</h${n}>`)
      i++
      continue
    }
    // Lists. A new item starts on a marker line; subsequent indented,
    // non-blank, non-marker lines are wrapped continuations of that item.
    const ulMarker = /^\s*[-*]\s+(.*)$/
    const olMarker = /^\s*\d+\.\s+(.*)$/
    const collectList = (marker, tag) => {
      const items = []
      while (i < lines.length) {
        const m = marker.exec(cur())
        if (m) {
          items.push(m[1])
          i++
        } else if (items.length && cur().trim() !== '' && /^\s+\S/.test(cur())) {
          items[items.length - 1] += ' ' + cur().trim()
          i++
        } else {
          break
        }
      }
      out.push(`<${tag}>${items.map((t) => `<li>${inline(t)}</li>`).join('')}</${tag}>`)
    }
    if (ulMarker.test(line)) {
      collectList(ulMarker, 'ul')
      continue
    }
    if (olMarker.test(line)) {
      collectList(olMarker, 'ol')
      continue
    }
    if (/^>\s?/.test(line)) {
      const buf = []
      while (i < lines.length && /^>\s?/.test(cur())) {
        buf.push(cur().replace(/^>\s?/, ''))
        i++
      }
      out.push(`<blockquote>${inline(buf.join(' '))}</blockquote>`)
      continue
    }
    if (line.trim() === '') {
      i++
      continue
    }
    const buf = []
    while (
      i < lines.length &&
      cur().trim() !== '' &&
      !/^(```|#|>|\s*[-*]\s+|\s*\d+\.\s+)/.test(cur())
    ) {
      buf.push(cur())
      i++
    }
    out.push(`<p>${inline(buf.join(' '))}</p>`)
  }
  return out.join('\n')
}

// ── frontmatter ─────────────────────────────────────────────────────────────
function parseFrontmatter(raw) {
  const text = raw.replace(/^﻿/, '')
  if (!text.startsWith('---')) return { data: {}, body: text }
  const end = text.indexOf('\n---', 3)
  if (end === -1) return { data: {}, body: text }
  const fmBlock = text.slice(3, end).trim()
  const body = text.slice(text.indexOf('\n', end + 1) + 1)
  const data = {}
  for (const lineRaw of fmBlock.split('\n')) {
    const line = lineRaw.trim()
    if (!line) continue
    const idx = line.indexOf(':')
    if (idx === -1) continue
    const key = line.slice(0, idx).trim()
    const val = line.slice(idx + 1).trim()
    if (val === 'null') {
      data[key] = null
    } else if (/^".*"$/.test(val) || /^'.*'$/.test(val)) {
      data[key] = val.slice(1, -1)
    } else if (/^-?\d+(\.\d+)?$/.test(val)) {
      data[key] = Number(val)
    } else {
      data[key] = val
    }
  }
  return { data, body }
}

function read(p) {
  return existsSync(p) ? readFileSync(p, 'utf8') : ''
}

// A lesson dir holds exactly one `.sail` — the whole lesson, `<controller>` block
// and template together. That file IS what the student edits and what the
// playground compiles; there is no second file to reconcile.
function lessonSources(dir) {
  if (!existsSync(dir)) return { stem: '', source: '' }
  const sail = readdirSync(dir).find((f) => f.endsWith('.sail'))
  if (!sail) return { stem: '', source: '' }
  return { stem: sail.replace(/\.sail$/, ''), source: read(join(dir, sail)) }
}

// ── collect ─────────────────────────────────────────────────────────────────
// The prose/code fields embed `import '@ship/…'` / `from '@ship/…'` SAMPLES.
// deck-bundler byte-scans every emitted JS chunk for those keyword+specifier
// shapes and FAILS the build on any bare `@ship/*` it can't resolve — it can't
// tell a real import from one sitting inside a string. So we base64-encode the
// heavy fields here and decode them at module load (see the emitted `dec`),
// keeping all import-like text out of the bundled output.
const enc = (s) => Buffer.from(String(s), 'utf8').toString('base64')

const slugs = readdirSync(PROSE)
  .filter((f) => f.endsWith('.md'))
  .map((f) => f.replace(/\.md$/, ''))
const lessons = []
for (const slug of slugs) {
  const { data, body } = parseFrontmatter(read(join(PROSE, `${slug}.md`)))
  const starter = lessonSources(join(LESSONS, slug, 'starter'))
  const solved = lessonSources(join(LESSONS, slug, 'solved'))
  lessons.push({
    slug,
    title: String(data.title ?? slug),
    description: String(data.description ?? ''),
    chapter: String(data.chapter ?? 'misc'),
    order: typeof data.order === 'number' ? data.order : 0,
    prev: data.prev == null ? null : String(data.prev),
    next: data.next == null ? null : String(data.next),
    solvedTag: solved.stem,
    starterStem: starter.stem,
    solvedStem: solved.stem,
    proseHtml: enc(renderMarkdown(body.trim())),
    starterSource: enc(starter.source),
    solvedSource: enc(solved.source),
  })
}
lessons.sort((a, b) => a.order - b.order)

// ── group by chapter (chapters ordered by their first lesson's order) ───────
const CHAPTER_TITLES = {
  surface: 'The surface',
  state: 'State and data',
  system: 'The system',
  agents: 'The agent loop',
}
const cap = (s) => s.charAt(0).toUpperCase() + s.slice(1)
const groupOrder = []
const byChapter = new Map()
for (const l of lessons) {
  if (!byChapter.has(l.chapter)) {
    byChapter.set(l.chapter, [])
    groupOrder.push(l.chapter)
  }
  byChapter.get(l.chapter).push({
    slug: l.slug,
    title: l.title,
    description: l.description,
    order: l.order,
    href: `/sail/tutorial/${l.slug}`,
  })
}
const groups = groupOrder.map((chapter) => ({
  chapter,
  title: CHAPTER_TITLES[chapter] ?? cap(chapter),
  lessons: byChapter.get(chapter),
}))

// ── emit ────────────────────────────────────────────────────────────────────
const banner = `// @generated by scripts/gen-tutorial.mjs — DO NOT EDIT.
// Source: packages/sail/tutorial/src/{prose,lessons}/<slug>. Regenerate with
// \`node scripts/gen-tutorial.mjs\` from apps/sail-site.
`

const types = `export interface TutorialLesson {
  slug: string
  title: string
  description: string
  chapter: string
  order: number
  prev: string | null
  next: string | null
  /** Custom-element tag of the solved lesson (its .sail stem). */
  solvedTag: string
  starterStem: string
  solvedStem: string
  proseHtml: string
  /** The starter \`.sail\` — controller block and template, as authored. */
  starterSource: string
  /** The solved \`.sail\`. */
  solvedSource: string
}

export interface TutorialIndexLesson {
  slug: string
  title: string
  description: string
  order: number
  href: string
}

export interface TutorialGroup {
  chapter: string
  title: string
  lessons: TutorialIndexLesson[]
}
`

// UTF-8-safe base64 decode (browser `atob` + Node global `atob`). The heavy
// fields below are stored encoded; we decode them once at module load.
const decoder = `// base64 → UTF-8 string. The prose/code fields are stored encoded so the
// bundler's bare-import scanner never sees the @ship-pack import SAMPLES they
// contain (those are display strings inside the lessons, not real imports).
function dec(b: string): string {
  const bin = atob(b)
  const bytes = new Uint8Array(bin.length)
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i)
  return new TextDecoder().decode(bytes)
}
`

const rawJson = JSON.stringify(lessons, null, 2)

const file =
  banner +
  '\n' +
  types +
  '\n' +
  decoder +
  '\n' +
  `const RAW_LESSONS: TutorialLesson[] = ${rawJson}

export const TUTORIAL_LESSONS: TutorialLesson[] = RAW_LESSONS.map((l) => ({
  ...l,
  proseHtml: dec(l.proseHtml),
  starterSource: dec(l.starterSource),
  solvedSource: dec(l.solvedSource),
}))

export const TUTORIAL_GROUPS: TutorialGroup[] = ${JSON.stringify(groups, null, 2)}

export function lessonBySlug(slug: string): TutorialLesson | null {
  return TUTORIAL_LESSONS.find((l) => l.slug === slug) ?? null
}
`

writeFileSync(OUT, file)
console.log(`gen-tutorial: ${lessons.length} lessons → ${OUT}`)
