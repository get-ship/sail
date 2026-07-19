// Context-aware completions for .sail templates.
//
// All closed vocabulary is read from the compiler's tables via WASM; no
// hand-maintained completion lists live here. The trigger shape is preserved
// for CodeMirror consumers, but every item set comes from
// `@ship/sail-compiler-wasm`'s `completion_vocabulary` export.

import initWasm, { completion_vocabulary } from '@ship/sail-compiler-wasm'

export interface SuiCompletionSource {
  trigger: string
  items: Array<{
    label: string
    detail?: string
    info?: string
    insert?: string
  }>
}

interface WasmVocabulary {
  events: string[]
  modifiers: Array<{
    canonical: string
    alias?: string
    arg?: string
    summary: string
  }>
  bind_targets: Array<{ target: string; summary: string }>
  blocks: Array<{ open: string; clauses: string[]; summary: string }>
}

const WASM_URL = '/sail_compiler_bg.wasm'

// Initialize WASM once at module load. The consumers of this package import
// it dynamically and await the module, so the init cost is paid before any
// completion function is called.
let vocab: WasmVocabulary | null = null
let initDone: Promise<void> | null = null

function ensureInit(): Promise<void> {
  if (!initDone) {
    initDone = (async () => {
      await initWasm(WASM_URL)
      vocab = completion_vocabulary() as WasmVocabulary
    })()
  }
  return initDone
}

await ensureInit()

function blockName(open: string): string {
  return open.replace('{#', '').replace('}', '')
}

function clauseName(clause: string): string {
  return clause.replace('{:', '').replace('}', '').trim()
}

function buildSources(): Record<string, SuiCompletionSource> {
  if (!vocab) throw new Error('sail-lsp-browser completions: WASM not initialized')

  const sources: Record<string, SuiCompletionSource> = {}

  sources['on:'] = {
    trigger: 'on:',
    items: vocab.events.map((e) => ({ label: e, detail: 'event' })),
  }

  sources['|'] = {
    trigger: '|',
    items: vocab.modifiers.flatMap((m) => {
      const base = { label: m.canonical, detail: 'modifier', info: m.summary }
      const alias = m.alias
        ? { label: m.alias, detail: `modifier alias (${m.canonical})`, info: m.summary }
        : []
      const examples: SuiCompletionSource['items'] = []
      if (m.arg) {
        if (m.canonical === 'key') {
          examples.push(
            { label: 'key:Enter', detail: 'key filter', insert: 'key:Enter' },
            { label: 'key:Escape', detail: 'key filter', insert: 'key:Escape' },
          )
        } else if (m.canonical === 'debounce' || m.canonical === 'throttle') {
          examples.push({
            label: `${m.canonical}:150`,
            detail: 'modifier',
            insert: `${m.canonical}:150`,
            info: m.summary,
          })
        }
      }
      return [base, alias, ...examples].filter((x): x is { label: string; detail?: string; info?: string; insert?: string } => !!x)
    }),
  }

  sources['bind:'] = {
    trigger: 'bind:',
    items: vocab.bind_targets.map((b) => ({
      label: b.target,
      detail: 'bind target',
      info: b.summary,
    })),
  }

  sources['{#'] = {
    trigger: '{#',
    items: vocab.blocks.map((b) => ({
      label: blockName(b.open),
      detail: 'block',
      info: b.summary,
    })),
  }

  sources['{:'] = {
    trigger: '{:',
    items: vocab.blocks.flatMap((b) =>
      b.clauses.map((c) => ({
        label: clauseName(c),
        detail: 'clause',
        info: b.summary,
      })),
    ),
  }

  return sources
}

const SOURCES = buildSources()

/** Pick the right completion source for the cursor position. Looks
 *  back at most 8 characters for a trigger string. Returns null if
 *  nothing recognizable precedes the cursor — caller can then fall
 *  back to the editor's built-in completion harness. */
export function completionsAt(source: string, offset: number): SuiCompletionSource | null {
  const safe = Math.max(0, Math.min(offset, source.length))
  const lookback = source.slice(Math.max(0, safe - 8), safe)
  // Longest triggers first so 'bind:' wins over ':' etc.
  for (const trigger of ['{#', '{:', 'bind:', 'on:', '|']) {
    if (lookback.endsWith(trigger)) return SOURCES[trigger]!
  }
  return null
}

/** All sources — what the editor wants if it's going to wire static
 *  triggers and let CodeMirror handle the dispatch itself. */
export function allSources(): SuiCompletionSource[] {
  return Object.values(SOURCES)
}
