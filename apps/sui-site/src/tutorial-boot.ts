// Tutorial playground machinery — the FFI seams ONLY. This module owns the
// things a SUI template cannot be: the wasm compiler call, the sandboxed
// preview iframe's postMessage protocol, and the theme-attribute observer.
//
// It does not touch page DOM: playground UI state (active tab, status,
// console lines, editor values) lives as signals in the route's <controller>,
// and the template reads it.

/** The wasm compile result the engine returns (mirrors @ship/sui-viewer). */
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
const SCENARIO_SLUGS = new Set(['22-data-live', '23-while-loading', '24-morphs'])

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

/** Compile the template, strip the controller, and post the run into the
 *  preview iframe. Pure seam: no status writes, no logging — the caller owns
 *  that state and reacts to the returned result. */
export function runInPreview(
  compile: CompileFn,
  iframe: HTMLIFrameElement,
  opts: { tag: string; template: string; controllerTs: string },
): CompileResult {
  const result = compile(opts.template, {
    componentName: opts.tag,
    shadow: false,
    setupUrl: `./${opts.tag}.controller.ts`,
  })
  if (result.kind === 'err') return result
  iframe.contentWindow?.postMessage(
    { kind: 'tutorial:run', template: result.js, controller: tsToEsm(opts.controllerTs), tag: opts.tag },
    '*',
  )
  return result
}

/** (Re)load the preview host page, matching the reader's resolved theme. */
export function reloadPreview(iframe: HTMLIFrameElement): void {
  const base = iframe.dataset.baseSrc ?? '/tut-host.html'
  const mode = document.documentElement.dataset.mode === 'dark' ? 'dark' : 'light'
  iframe.src = `${base}?mode=${mode}&_=${Date.now()}`
}

/** Push a lesson's mock-daemon scenario into the ready iframe, if it has one. */
export async function pushScenario(iframe: HTMLIFrameElement, slug: string): Promise<void> {
  if (!SCENARIO_SLUGS.has(slug)) return
  try {
    const scenarioUrl = `/tut-${slug}.js`
    const head = await fetch(scenarioUrl, { method: 'HEAD' })
    if (!head.ok) return
    const mod = (await import(/* @vite-ignore */ scenarioUrl)) as { default: Record<string, unknown> }
    iframe.contentWindow?.postMessage({ kind: 'tutorial:scenario', scenario: mod.default }, '*')
    await new Promise((r) => setTimeout(r, 80))
  } catch (err) {
    console.warn('[sui-site/tutorial] scenario load skipped:', err)
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
  window.addEventListener('message', onMessage)
  return () => window.removeEventListener('message', onMessage)
}

/** The reader's theme lives on <html data-theme/data-mode> — an external DOM
 *  channel, so a MutationObserver is the honest seam. Returns dispose. */
export function observeTheme(onChange: () => void): () => void {
  const obs = new MutationObserver(onChange)
  obs.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ['data-mode', 'data-theme'],
  })
  return () => obs.disconnect()
}

/** Imperative surface of an upgraded <sui-code-editor> (its published API). */
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
