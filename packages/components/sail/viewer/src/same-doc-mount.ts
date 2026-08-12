// Same-document preview mount for tutorial + free playground.
//
// Framework-owned substrate seam (SPEC-PLATFORM-NEUTRALITY-1): script inject,
// host/waiter registry, and style injection live here so app compositions stay
// Pure (no document/window). Apps call `runInHost` only.
//
// Owns the wasm compile call and same-document preview mount
// (compile → inline module → __mount into a host).
//
// Preview is same-document: the surface shares the parent page's Sail
// runtime, customElements registry, and theme/token CSS. No iframe, no
// postMessage protocol, no parallel CSS concat.
//
// CSP note: deck serves `script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'`
// — no `unsafe-eval`, no `blob:`. So we cannot use `new Function` or blob
// module URLs. Inline `<script type="module">` (allowed by 'unsafe-inline')
// plus the page importmap is the mount path that works under that policy.

import { debounce, type SailHost } from '@ship/sail-runtime'
import { unwrapSailHost } from '@ship/sail-runtime/internal'

/** The wasm compile result the engine returns. */
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
// Mirrors crates/sail/compiler/src/runtime_ambient.rs (S5.1 + S5.2). Keep in
// lockstep so lesson controllers that drop package imports still inject.
const RUNTIME_AMBIENT: Array<[string, string]> = [
  ['__invalidate', '__invalidate'],
  ['applyTheme', 'applyTheme'],
  ['batch', 'batch'],
  ['controller', 'controller'],
  ['debounce', 'debounce'],
  ['defineStore', 'defineStore'],
  ['derived', 'derived'],
  ['effect', 'effect'],
  ['filteredList', 'filteredList'],
  ['form', 'form'],
  ['interval', 'interval'],
  ['mount', 'mountAmbient'],
  ['mutation', 'mutation'],
  ['navigate', 'navigate'],
  ['onCleanup', 'onCleanup'],
  ['persistTheme', 'persistTheme'],
  ['provideStore', 'provideStore'],
  ['query', 'query'],
  ['readStoredTheme', 'readStoredTheme'],
  ['signal', 'signal'],
  ['theme', 'theme'],
  ['timeline', 'timeline'],
  ['untrack', 'untrack'],
  ['useStore', 'useStore'],
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
      bound.add(as ? as[1]! : t.split(/\s+/)[0]!)
    }
  }
  for (const m of body.matchAll(/\b(?:const|let|var|function|class)\s+(\w+)/g)) {
    bound.add(m[1]!)
  }
  const needed: string[] = []
  for (const [authored, exported] of RUNTIME_AMBIENT) {
    if (bound.has(authored)) continue
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
      .map((p) => p.split(':')[0]!.trim())
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

/**
 * Module-private host/waiter registry (same-document only). Prefer this over
 * ambient window slots so product apps never name god-globals.
 */
const tutWaiters: Record<string, TutWaiter> = {}
const tutHosts: Record<string, any> = {}

function ensureWaiters(): Record<string, TutWaiter> {
  return tutWaiters
}

function ensureHosts(): Record<string, any> {
  return tutHosts
}

/**
 * Build one ESM module source that:
 *  1. runs the lesson controller (export default → const __tutSetup)
 *  2. rewrites the compiled template's `import setup from "…"` to that const
 *  3. mounts the view into the host and resolves a waiter via the runtime API
 *
 * Relative lesson mock clients are rewritten onto package exports so the page
 * importmap / bundler graph can resolve them (no blob: URLs — CSP forbids them).
 *
 * The mount module imports `@ship/sail-viewer/same-doc-mount` for the registry
 * so it does not touch window/document for host lookup.
 */
function buildMountModule(opts: {
  runId: string
  controllerSrc: string
  templateJs: string
  css: string | null | undefined
}): string {
  let controller = opts.controllerSrc
    ? wrapControllerBody(tsToEsm(opts.controllerSrc))
    : 'const __tutSetup = { setup: () => ({}) };'
  if (opts.controllerSrc) {
    // Lesson mock clients live under app FileKind Composition home
    // (`@/compositions/*` / project_map SSOT). Rewrite relative or alias forms
    // onto that specifier so bundler + future importmap wiring share one shape.
    controller = controller.replace(
      /from\s*["'][^"']*compositions\/([^"']+)["']/g,
      (_m, bare: string) => {
        const key = bare.replace(/\.ts$/, '')
        return `from ${JSON.stringify(`@/compositions/${key}`)}`
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
  const runIdLit = JSON.stringify(opts.runId)

  // Injected module uses the runtime registry API (no ambient god-globals).
  return `import { takeTutMount } from ${JSON.stringify('@ship/sail-viewer/same-doc-mount')};
${controller}

${template}

;(async () => {
  const runId = ${runIdLit}
  const { host, waiter } = takeTutMount(runId)
  try {
    if (!host) throw new Error('preview host not found')
    while (host.firstChild) host.removeChild(host.firstChild)
    const css = ${cssLit}
    let styleEl = null
    if (css) {
      styleEl = host.ownerDocument.createElement('style')
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
  }
})();
`
}

/**
 * Consume host + waiter registration for a runId (called from the injected
 * mount module). Removes entries so re-runs do not leak.
 */
export function takeTutMount(runId: string): {
  host: any | undefined
  waiter: TutWaiter | undefined
} {
  const host = tutHosts[runId]
  const waiter = tutWaiters[runId]
  delete tutHosts[runId]
  delete tutWaiters[runId]
  return { host, waiter }
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
  host: SailHost,
  opts: { tag: string; source: string },
  prev?: PreviewMount | null,
): { result: CompileResult; mount: PreviewMount | null; ready?: Promise<PreviewMount | null> } {
  // Framework unwrap: SailHost is the opaque author surface; web port is any.
  const hostEl = unwrapSailHost(host)
  const result = compile(opts.source, {
    setupUrl: SETUP_MARKER,
    setupMarker: SETUP_MARKER,
  })
  if (result.kind === 'err') return { result, mount: prev ?? null }

  const runId = `tut-run-${++runSeq}`

  // Tear prior mount before registering the next host.
  try {
    prev?.dispose()
  } catch {
    /* best-effort */
  }
  if (activeScript) {
    activeScript.remove()
    activeScript = null
  }

  // Register host by runId — mount module looks it up via takeTutMount.
  const hosts = ensureHosts()
  hosts[runId] = hostEl
  hostEl.setAttribute('data-tut-host', runId)

  const ctrlBody = extractController(opts.source)
  const moduleSrc = buildMountModule({
    runId,
    controllerSrc: ctrlBody,
    templateJs: result.js,
    css: result.css,
  })

  const waiters = ensureWaiters()
  const ready = new Promise<PreviewMount>((resolve, reject) => {
    waiters[runId] = { resolve, reject }
  })

  const doc = hostEl.ownerDocument
  const script = doc.createElement('script')
  script.type = 'module'
  script.setAttribute('data-tut-mount', runId)
  script.textContent = moduleSrc
  activeScript = script
  doc.head.appendChild(script)

  const mountPromise = ready
    .then((m) => m)
    .catch((err) => {
      const message = err instanceof Error ? err.message : String(err)
      throw new Error(message)
    })

  const clearHostReg = (): void => {
    if (hosts[runId] === hostEl) delete hosts[runId]
    if (hostEl.getAttribute('data-tut-host') === runId) {
      hostEl.removeAttribute('data-tut-host')
    }
    delete waiters[runId]
  }

  // Synchronous return for compile-ok; page controller awaits ready when present.
  const mount: PreviewMount = {
    dispose: () => {
      void mountPromise
        .then((m) => m.dispose())
        .catch(() => {})
      script.remove()
      if (activeScript === script) activeScript = null
      clearHostReg()
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
          clearHostReg()
        }
        return mount
      })
      .catch((err) => {
        script.remove()
        if (activeScript === script) activeScript = null
        clearHostReg()
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
export interface CodeEditorHost extends any {
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
