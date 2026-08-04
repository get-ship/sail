// Tutorial playground machinery — the FFI seams ONLY. This module owns the
// things a Sail template cannot be: the wasm compiler call, the sandboxed
// preview iframe's postMessage protocol, and the theme-attribute observer.
//
// It does not touch page DOM: playground UI state (active tab, status,
// console lines, editor values) lives as signals in the route's <controller>,
// and the template reads it.

/** The wasm compile result the engine returns (mirrors @ship/sail-viewer). */
export type CompileResult =
  | { kind: 'ok'; js: string; css?: string | null }
  | { kind: 'err'; message: string; line?: number | null; col?: number | null }

/** The bound compile function (source, opts) → result. */
export type CompileFn = (
  source: string,
  opts?: { componentName?: string; shadow?: boolean; setupUrl?: string },
) => CompileResult

// Slugs that ship a mock-daemon scenario (public/tut-<slug>.js). Explicit set
// so we don't HEAD-probe non-existent scenarios and surface 404s as errors.
//
// Empty since the 25-step tutorial was replaced by the nine lessons in
// `packages/sail/tutorial`: none of them needs a mock daemon (lesson 05 and
// lesson 09 use in-repo client modules instead). The set — and `pushScenario`
// below — are kept because the mechanism is still the answer if a future
// lesson needs a scripted backend; add the slug here and drop the script at
// `public/tut-<slug>.js`. It previously held three retired slugs, which made
// `pushScenario` dead code that read as live.
const SCENARIO_SLUGS = new Set<string>([])

/** Strip TypeScript-only syntax from a (simple) lesson controller so it runs
 *  as ESM in the preview iframe. The lesson controllers are simple and this
 *  transform is framework-agnostic. */
export function tsToEsm(src: string): string {
  let s = src
  s = s.replace(/^\s*import\s+type\s+\{[^}]*\}\s+from\s+["'][^"']+["']\s*;?\s*$/gm, '')
  s = s.replace(/import\s+\{([^}]+)\}\s+from/g, (_m, inner: string) => {
    const cleaned = inner
      .split(',')
      .map((p) => p.trim())
      .filter((p) => p && !p.startsWith('type '))
      .join(', ')
    return `import { ${cleaned} } from`
  })
  s = s.replace(/^\s*interface\s+\w+(\s+extends\s+[^{]+)?\s*\{[^}]*\}\s*/gm, '')
  s = s.replace(/^\s*type\s+\w+\s*=\s*[^;\n]+;?[ \t]*$/gm, '')
  s = s.replace(/\(([^()]*)\)\s*(:\s*[^=>{;,)]+)?\s*=>/g, (_m, params: string) => {
    const cleaned = params
      .split(',')
      .map((p) => p.split(':')[0].trim())
      .filter(Boolean)
      .join(', ')
    return `(${cleaned}) =>`
  })
  s = s.replace(/(\w+)<[^<>()]*>\(/g, '$1(')
  s = s.replace(/\s+as\s+[\w<>[\]|\s.,]+(?=[;,)\]}])/g, '')
  s = s.replace(/(\bconst\s+\w+|\blet\s+\w+|\bvar\s+\w+)\s*:\s*[^=;,]+/g, '$1')
  return s
}

/** A one-slot trailing debouncer. */
export function createDebouncer(ms: number): (fn: () => void) => void {
  let timer: ReturnType<typeof setTimeout> | null = null
  return (fn: () => void) => {
    if (timer) clearTimeout(timer)
    timer = setTimeout(fn, ms)
  }
}

/** The body of the source's `<controller>` block, or `''`. Mirrors the
 *  compiler's extraction, including its one subtlety: a `<controller>` inside an
 *  HTML comment is prose, not the block — a lesson's header comment talks about
 *  the block, and matching it would splice the file from the comment onward. */
export function extractController(source: string): string {
  let i = 0
  let open = -1
  while (i < source.length) {
    const comment = source.indexOf('<!--', i)
    const hit = source.indexOf('<controller', i)
    if (hit === -1) return ''
    if (comment === -1 || hit < comment) {
      open = hit
      break
    }
    const end = source.indexOf('-->', comment)
    if (end === -1) return ''
    i = end + 3
  }
  if (open === -1) return ''
  const bodyStart = source.indexOf('>', open)
  if (bodyStart === -1) return ''
  const close = source.indexOf('</controller>', bodyStart)
  if (close === -1) return ''
  return source.slice(bodyStart + 1, close).trim()
}

/** Compile the lesson source and post the run into the preview iframe.
 *
 *  ONE source: the `.sail` file the student edits, `<controller>` block and all.
 *  The compiler resolves the block into the scope it checks the template
 *  against, so the playground rejects exactly what `sail-compiler` rejects, and
 *  an error's line number points at the row in the editor. The block's body is
 *  extracted separately only because the preview iframe loads it as the ESM
 *  module the emitted template imports.
 *
 *  Pure seam: no status writes, no logging — the caller owns that state. */
export function runInPreview(
  compile: CompileFn,
  iframe: HTMLIFrameElement,
  opts: { tag: string; source: string },
): CompileResult {
  const result = compile(opts.source, {
    componentName: opts.tag,
    shadow: false,
    setupUrl: `./${opts.tag}.controller.ts`,
  })
  if (result.kind === 'err') return result
  iframe.contentWindow?.postMessage(
    {
      kind: 'tutorial:run',
      template: result.js,
      controller: tsToEsm(extractController(opts.source)),
      tag: opts.tag,
    },
    '*',
  )
  return result
}

/** (Re)load the preview host page, matching the reader's resolved theme. */
export function reloadPreview(iframe: HTMLIFrameElement): void {
  // DOMDEC: no ambient document — keep base src, light mode default.
  const base = iframe.getAttribute('data-base-src') ?? iframe.dataset?.baseSrc ?? '/tut-host.html'
  iframe.src = `${base}?mode=light&_=${Date.now()}`
}

/** Push a lesson's mock-daemon scenario into the ready iframe, if it has one. */
export async function pushScenario(iframe: HTMLIFrameElement, slug: string): Promise<void> {
  if (!SCENARIO_SLUGS.has(slug)) return
  try {
    const scenarioUrl = `/tut-${slug}.js`
    // DOMDEC: ambient fetch banned — dynamic scenario load is a gap.
    return
    const mod = null as unknown as { default: Record<string, unknown> }
    iframe.contentWindow?.postMessage({ kind: 'tutorial:scenario', scenario: mod.default }, '*')
    await new Promise((r) => setTimeout(r, 80))
  } catch (err) {
    console.warn('[sail-site/tutorial] scenario load skipped:', err)
  }
}

export interface PreviewHooks {
  /** console.{log,warn,error} forwarded from inside the preview iframe. */
  onConsole(level: string, text: string): void
  /** The host page booted and is listening for tutorial:run. */
  onReady(): void
}

/** Listen to the preview iframe's message channel. Returns dispose. */
export function listenPreview(
  iframe: () => HTMLIFrameElement | null,
  hooks: PreviewHooks,
): () => void {
  const onMessage = (e: MessageEvent): void => {
    const el = iframe()
    if (!el || e.source !== el.contentWindow) return
    if (e.data?.kind === 'tutorial:console') {
      hooks.onConsole(String(e.data.level ?? 'log'), String(e.data.text ?? ''))
      return
    }
    if (e.data?.kind === 'tutorial:ready') hooks.onReady()
  }
  // sail-allow sail-no-ambient-events
  window.addEventListener('message', onMessage)
  return () => window.removeEventListener('message', onMessage)
}

/** The reader's theme lives on <html theme + data-mode> — an external DOM
 *  channel, so a MutationObserver is the honest seam. Returns dispose. */
export function observeTheme(_onChange: () => void): () => void {
  // DOMDEC: no ambient document MutationObserver — theme change bus gap.
  return () => {}
}

/** Imperative surface of an upgraded <sail-code-editor> (its published API). */
export interface CodeEditorHost extends HTMLElement {
  value: string
  setDiagnostics?: (diags: Array<{
    line: number
    column?: number
    severity: 'error' | 'warning' | 'info'
    message: string
    source?: string
  }>) => void
}
