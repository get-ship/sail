// Live Sail boot — shared FFI seams for tutorial + free playground.
// Owns the wasm compile call and same-document preview mount
// (compile → inline module → __mount into a host).
//
// Does not own page chrome: editor values, status, console lines, and tabs
// live as signals in each route's <controller>; the template reads them.
//
// Preview is same-document: the surface shares the parent page's Sail
// runtime, customElements registry, and theme/token CSS. No iframe, no
// postMessage protocol, no parallel CSS concat.
//
// CSP note: deck serves `script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'`
// — no `unsafe-eval`, no `blob:`. So we cannot use `new Function` or blob
// module URLs. Inline `<script type="module">` (allowed by 'unsafe-inline')
// plus the page importmap is the mount path that works under that policy.

import { debounce } from '@ship/sail-runtime'

/** The wasm compile result the engine returns (mirrors @ship/sail-viewer). */
export type CompileResult =
  | { kind: 'ok'; js: string; css?: string | null }
  | { kind: 'err'; message: string; line?: number | null; col?: number | null }

/** The bound compile function (source, opts) → result. */
export type CompileFn = (
  source: string,
  opts?: {
    mode?: 'view' | 'component' | 'auto'
    componentName?: string
    shadow?: boolean
    setupUrl?: string
    setupMarker?: string
  },
) => CompileResult

/** Marker the compiler stamps as `import setup from "<marker>"`; rewritten at mount. */
const SETUP_MARKER = './__tutorial__.controller.ts'

/**
 * Ambient controller vocabulary (mirrors crates/sail/compiler runtime_ambient).
 * Inline `<controller>` bodies use these without imports; the real emit path
 * injects them via wrap_controller_body. We do the same before inlining.
 * Each entry: [authored, exported] — aliased when they differ (mount).
 */
const RUNTIME_AMBIENT: Array<[string, string]> = [
  ['batch', 'batch'],
  ['controller', 'controller'],
  ['derived', 'derived'],
  ['effect', 'effect'],
  ['mount', 'mountAmbient'],
  ['onCleanup', 'onCleanup'],
  ['signal', 'signal'],
  ['untrack', 'untrack'],
]
const RUNTIME_SPECIFIER = '@ship/sail-runtime/internal'

/** Inject ambient runtime imports a lesson controller references but does not bind. */
function wrapControllerBody(body: string): string {
  const bound = new Set<string>()
  for (const m of body.matchAll(/\bimport\s*\{([^}]+)\}\s*from/g)) {
    for (const part of m[1].split(',')) {
      const t = part.trim()
      if (!t) continue
      const as = /\bas\s+(\w+)\s*$/.exec(t)
      bound.add(as ? as[1] : t.split(/\s+/)[0]!)
    }
  }
  for (const m of body.matchAll(/\b(?:const|let|var|function|class)\s+(\w+)/g)) {
    bound.add(m[1]!)
  }
  const needed: string[] = []
  for (const [authored, exported] of RUNTIME_AMBIENT) {
    if (bound.has(authored)) continue
    // word-boundary reference (not already part of a longer ident)
    if (!new RegExp(`\\b${authored}\\b`).test(body)) continue
    needed.push(authored === exported ? authored : `${exported} as ${authored}`)
  }
  if (needed.length === 0) return body
  return `import { ${needed.join(', ')} } from '${RUNTIME_SPECIFIER}';\n${body}`
}

/** Strip TypeScript-only syntax from a (simple) lesson controller so it runs
 *  as plain ESM. The lesson controllers are simple and this transform is
 *  framework-agnostic. */
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

/** A one-slot trailing debouncer (lifecycle-bound when called under a setup scope). */
export function createDebouncer(ms: number): (fn: () => void) => void {
  let pending: (() => void) | null = null
  const fire = debounce(() => {
    const f = pending
    pending = null
    f?.()
  }, ms)
  return (fn: () => void) => {
    pending = fn
    fire()
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

export interface PreviewMount {
  dispose: () => void
}

type TutWaiter = {
  resolve: (v: { dispose: () => void }) => void
  reject: (e: unknown) => void
}

declare global {
  interface Window {
    __tutWaiters?: Record<string, TutWaiter>
  }
}

function ensureWaiters(): Record<string, TutWaiter> {
  if (!window.__tutWaiters) window.__tutWaiters = {}
  return window.__tutWaiters
}

/**
 * Build one ESM module source that:
 *  1. runs the lesson controller (export default → const __tutSetup)
 *  2. rewrites the compiled template's `import setup from "…"` to that const
 *  3. mounts the view into the host and resolves a waiter on window
 *
 * Relative lesson mock clients are rewritten onto package exports so the page
 * importmap / bundler graph can resolve them (no blob: URLs — CSP forbids them).
 */
function buildMountModule(opts: {
  runId: string
  hostAttr: string
  controllerSrc: string
  templateJs: string
  css: string | null | undefined
}): string {
  let controller = opts.controllerSrc
    ? wrapControllerBody(tsToEsm(opts.controllerSrc))
    : 'const __tutSetup = { setup: () => ({}) };'
  if (opts.controllerSrc) {
    // Relative compositions → package export paths the app can resolve.
    controller = controller.replace(
      /from\s*["'][^"']*compositions\/([^"']+)["']/g,
      (_m, bare: string) => {
        const key = bare.replace(/\.ts$/, '')
        return `from ${JSON.stringify(`@ship/sail-tutorial/compositions/${key}`)}`
      },
    )
    controller = controller.replace(/\bexport\s+default\s+/g, 'const __tutSetup = ')
    if (!/const __tutSetup\s*=/.test(controller)) {
      controller += '\nconst __tutSetup = { setup: () => ({}) };'
    }
  }

  let template = opts.templateJs.replace(
    /^\s*import\s+setup\s+from\s+["'][^"']+["']\s*;?\s*$/gm,
    'const setup = __tutSetup;',
  )
  // Capture the default export instead of re-exporting.
  template = template.replace(/\bexport\s+default\s+/g, 'const __tutDef = ')

  const cssLit = opts.css ? JSON.stringify(opts.css) : 'null'
  const hostAttrLit = JSON.stringify(opts.hostAttr)
  const runIdLit = JSON.stringify(opts.runId)

  return `${controller}

${template}

;(async () => {
  const runId = ${runIdLit}
  const hostAttr = ${hostAttrLit}
  const waiters = (window.__tutWaiters = window.__tutWaiters || {})
  const waiter = waiters[runId]
  try {
    const host = document.querySelector('[' + hostAttr + ']')
    if (!host) throw new Error('preview host not found')
    while (host.firstChild) host.removeChild(host.firstChild)
    const css = ${cssLit}
    let styleEl = null
    if (css) {
      styleEl = document.createElement('style')
      styleEl.setAttribute('data-tut-preview-css', '')
      styleEl.textContent = css
      host.appendChild(styleEl)
    }
    const def = __tutDef
    if (!def || typeof def.setup !== 'function') {
      throw new Error('compiled module has no setup')
    }
    const ctx = {
      route: { params: {}, path: '/', query: {}, hash: '' },
      navigate: () => {},
    }
    // Prefer a real route factory when the importmap has it; fall through.
    try {
      const routeMod = await import('@ship/sail-runtime/route')
      if (typeof routeMod.route === 'function') {
        ctx.route = routeMod.route({ url: new URL('http://localhost/') })
      }
    } catch { /* route is optional for lesson mounts */ }
    const scope = def.setup(ctx)
    if (!scope || typeof scope.__mount !== 'function') {
      throw new Error('setup() did not return __mount')
    }
    const before = new Set(Array.from(host.childNodes))
    const cleanup = scope.__mount(host)
    const dispose = () => {
      try { cleanup() } catch { /* best-effort */ }
      for (const n of Array.from(host.childNodes)) {
        if (!before.has(n) || n === styleEl) n.parentNode && n.parentNode.removeChild(n)
      }
      if (styleEl) styleEl.remove()
    }
    if (waiter) waiter.resolve({ dispose })
  } catch (err) {
    if (waiter) waiter.reject(err)
    else console.error('[tutorial-preview]', err)
  } finally {
    delete waiters[runId]
  }
})();
`
}

let runSeq = 0
let activeScript: HTMLScriptElement | null = null

/**
 * Compile the lesson source and mount it into a same-document host element.
 *
 * ONE source: the `.sail` file the student edits, `<controller>` block and all.
 * The compiler resolves the block into the scope it checks the template
 * against; the block body is inlined as the setup the emitted template uses.
 *
 * Compile errors return `{kind:'err'}` without disposing the prior mount
 * (last-good preview stays). A clean compile tears down the previous mount
 * before the new one appends.
 */
export function runInHost(
  compile: CompileFn,
  host: HTMLElement,
  opts: { tag: string; source: string },
  prev?: PreviewMount | null,
): { result: CompileResult; mount: PreviewMount | null; ready?: Promise<PreviewMount | null> } {
  const result = compile(opts.source, {
    setupUrl: SETUP_MARKER,
    setupMarker: SETUP_MARKER,
  })
  if (result.kind === 'err') return { result, mount: prev ?? null }

  const runId = `tut-run-${++runSeq}`
  const hostAttr = `data-tut-host="${runId}"`
  // Stamp the host so the inline module can find it without holding a closed-
  // over node reference across the module boundary.
  for (const a of Array.from(host.attributes)) {
    if (a.name.startsWith('data-tut-host')) host.removeAttribute(a.name)
  }
  host.setAttribute('data-tut-host', runId)

  const ctrlBody = extractController(opts.source)
  const moduleSrc = buildMountModule({
    runId,
    hostAttr: `data-tut-host="${runId}"`,
    controllerSrc: ctrlBody,
    templateJs: result.js,
    css: result.css,
  })

  const waiters = ensureWaiters()
  const ready = new Promise<PreviewMount>((resolve, reject) => {
    waiters[runId] = { resolve, reject }
  })

  // Tear prior mount only after we know compile succeeded.
  try {
    prev?.dispose()
  } catch {
    /* best-effort */
  }
  if (activeScript) {
    activeScript.remove()
    activeScript = null
  }

  const script = document.createElement('script')
  script.type = 'module'
  script.setAttribute('data-tut-mount', runId)
  script.textContent = moduleSrc
  activeScript = script
  document.head.appendChild(script)

  const mountPromise = ready
    .then((m) => m)
    .catch((err) => {
      const message = err instanceof Error ? err.message : String(err)
      throw new Error(message)
    })

  // Synchronous return for compile-ok; page controller awaits ready when present.
  const mount: PreviewMount = {
    dispose: () => {
      void mountPromise
        .then((m) => m.dispose())
        .catch(() => {})
      script.remove()
      if (activeScript === script) activeScript = null
      host.removeAttribute('data-tut-host')
    },
  }

  return {
    result,
    mount,
    ready: mountPromise
      .then((m) => {
        // Replace dispose with the real one once ready.
        mount.dispose = () => {
          try {
            m.dispose()
          } catch {
            /* best-effort */
          }
          script.remove()
          if (activeScript === script) activeScript = null
          host.removeAttribute('data-tut-host')
        }
        return mount
      })
      .catch((err) => {
        script.remove()
        if (activeScript === script) activeScript = null
        return Promise.reject(err)
      }),
  }
}

/** Install same-document console.{log,warn,error,info} capture. Returns dispose. */
export function installConsoleCapture(
  onConsole: (level: string, text: string) => void,
): () => void {
  const levels = ['log', 'warn', 'error', 'info'] as const
  const originals = new Map<string, (...args: unknown[]) => void>()
  for (const level of levels) {
    const orig = console[level].bind(console) as (...args: unknown[]) => void
    originals.set(level, orig)
    console[level] = ((...args: unknown[]) => {
      try {
        const text = args
          .map((a) => {
            if (typeof a === 'string') return a
            try {
              return JSON.stringify(a)
            } catch {
              return String(a)
            }
          })
          .join(' ')
        onConsole(level, text)
      } catch {
        /* forwarding is best-effort */
      }
      orig(...args)
    }) as typeof console.log
  }
  return () => {
    for (const level of levels) {
      const orig = originals.get(level)
      if (orig) (console[level] as unknown) = orig
    }
  }
}

/** Imperative surface of an upgraded <sail-code-editor> (its published API). */
export interface CodeEditorHost extends HTMLElement {
  value: string
  setDiagnostics?: (
    diags: Array<{
      line: number
      column?: number
      severity: 'error' | 'warning' | 'info'
      message: string
      source?: string
    }>,
  ) => void
}
