// View-mode mount (build-spec §Mount by raw .sail source). Wraps the harness
// `compileAndMount` view-mode branch over the INJECTED `compile()` (purity
// seam — the engine never imports a wasm URL or `fetch`):
//
//   1. compile(source) → { kind:'ok', js, css } | { kind:'err', … }
//   2. evalViewModule(js) → { setup }
//   3. ctx = { ship, route, navigate }; scope = def.setup(ctx)
//   4. cleanup = scope.__mount(stageEl); snapshot childNodes before so
//      dispose() empties the container (cleanup() + remove appended nodes).
//
// The stage is a plain container element living inside `sail-render`'s OWN
// shadow root (obtained via `bind:this={stageEl}`). Compiled CSS adopts onto
// that owning shadow root's `adoptedStyleSheets` (scoped, no global leak) and
// the inherited-property baseline is re-rooted from role tokens on the stage
// container so a host-page `font`/`color` does NOT bleed into the specimen
// (build-spec §Isolation decision). `sail-render`'s shadow IS the engine
// shadow root the spec describes — no nested shadow (a nested shadow would
// hide the specimen from the host's `shadowRoot.querySelector`).

import { route as makeRoute, type RouteSignals } from '@ship/sail-runtime/route'
import { evalViewModule, type InjectedSetup } from './eval-module'
import type { CompileFn } from '../../wasm-bootstrap'

export interface MountSourceOpts {
  source: string
  /** Injected ctx-bearing setup (view-mode). Defaults to none. */
  setup?: InjectedSetup
  /** Marker the compiler stamps as the setup import, rewritten to `setup`. */
  setupUrl?: string
  /** Flat named providers (same surface as mount inject). */
  providers?: Record<string, unknown>
  route?: RouteSignals
  navigate?: (path: string) => void
  /** Disposer for the prior mount, run AFTER a clean compile but BEFORE the
   *  new specimen mounts — so a compile error keeps the last-good specimen,
   *  and a clean recompile tears the old one down before the snapshot diff. */
  preDispose?: () => void
}

export interface ViewMount {
  dispose: () => void
}

// Inherited CSS properties a shadow boundary does NOT block. Re-rooted from
// role tokens at the stage so the host page's inherited styles don't leak in.
// NOT `all:initial` — that would kill token inheritance (build-spec).
const INHERITED_BASELINE: Record<string, string> = {
  'font-family': 'var(--font-body)',
  'font-size': 'var(--text-base)',
  'font-weight': 'var(--font-weight-normal)',
  'font-style': 'normal',
  'line-height': 'var(--leading-normal)',
  color: 'var(--foreground)',
  'letter-spacing': 'normal',
  'text-align': 'start',
  direction: 'ltr',
}

/**
 * Attach the engine shadow root on the host and relocate the compiled
 * light-DOM template into it (build-spec §Isolation — "an engine-created
 * shadow root on the stage host").
 *
 * The harness compiles `sail-render` with `shadow:false` (the wasm default),
 * so its template renders into the host's light DOM. The controller `mount`
 * runs AFTER that render, so the engine moves the already-wired template
 * subtree into a fresh shadow root — node identity is preserved, so every
 * binding (`{#if}` anchors, the `bind:this={stageEl}` ref) stays live. This
 * gives the specimen real shadow isolation + adopted-CSS scoping, and the
 * theme/density custom properties still inherit through the boundary.
 * Idempotent — a second call is a no-op once the shadow exists.
 */
export function attachEngineShadow(host: HTMLElement): ShadowRoot {
  if (host.shadowRoot) return host.shadowRoot
  const moved = Array.from(host.childNodes)
  const root = host.attachShadow({ mode: 'open' })
  for (const n of moved) root.appendChild(n)
  return root
}

/** Re-establish the inherited-property baseline on the stage container from
 *  role tokens, and return the owning shadow root (where compiled CSS adopts).
 *  Does NOT attach a nested shadow — the stage lives in `sail-render`'s own
 *  shadow, so the specimen stays reachable from `host.shadowRoot`. */
export function prepareStage(stageEl: HTMLElement): ShadowRoot | null {
  for (const [prop, value] of Object.entries(INHERITED_BASELINE)) {
    stageEl.style.setProperty(prop, value)
  }
  const root = stageEl.getRootNode()
  return root instanceof ShadowRoot ? root : null
}

/** Adopt a compiled CSS string onto the stage shadow root's
 *  `adoptedStyleSheets` (scoped; never injected into the document head).
 *  Returns the disposer that removes the adopted sheet. */
export function adoptCompiledCss(root: ShadowRoot, css: string | null): () => void {
  if (!css || typeof CSSStyleSheet === 'undefined') return () => {}
  let sheet: CSSStyleSheet
  try {
    sheet = new CSSStyleSheet()
    sheet.replaceSync(css)
    root.adoptedStyleSheets = [...root.adoptedStyleSheets, sheet]
  } catch {
    return () => {}
  }
  return () => {
    try {
      root.adoptedStyleSheets = root.adoptedStyleSheets.filter((s) => s !== sheet)
    } catch {
      /* best-effort */
    }
  }
}

/**
 * Compile + mount a raw `.sail` view-mode source into the stage container.
 * Throws if `compile()` returns `{kind:'err'}` (the controller catches it and
 * renders the compile-error branch). Mount throws (setup/`__mount`) propagate
 * so the controller's secondary try/catch renders a panel, not a blank stage.
 */
export function mountBySource(
  stageEl: HTMLElement,
  compile: CompileFn,
  opts: MountSourceOpts,
): { mount: ViewMount; js: string; css: string | null } {
  // Pass the setup marker under BOTH keys: the production wasm `makeCompile`
  // reads `setupUrl`; the Node harness `compileSui` reads `setupMarker`. Both
  // become the compiler's `setup_url`, emitting `import setup from "<marker>"`.
  const compiled = compile(
    opts.source,
    opts.setupUrl != null ? { setupUrl: opts.setupUrl, setupMarker: opts.setupUrl } : {},
  )
  if (compiled.kind === 'err') {
    const where = compiled.line != null ? ` (${compiled.line}:${compiled.col ?? 0})` : ''
    throw new Error(`sail-compiler: ${compiled.message}${where}`)
  }

  const def = evalViewModule(compiled.js, opts.setup)
  const providers = opts.providers ?? {}
  const ctx = {
    ...providers,
    ...(Object.keys(providers).length > 0 ? { providers } : {}),
    route: opts.route ?? makeRoute({ url: new URL('http://localhost/') }),
    navigate: opts.navigate ?? (() => {}),
  } as unknown as Parameters<typeof def.setup>[0]
  const scope = def.setup(ctx) as { __mount: (c: HTMLElement) => () => void }

  // Clean compile + setup: now safe to tear down the prior mount before the
  // new one appends (so the empties-snapshot diff doesn't catch new nodes).
  opts.preDispose?.()

  const root = prepareStage(stageEl)
  const before = new Set(Array.from(stageEl.childNodes))
  const cleanup = scope.__mount(stageEl)
  const disposeCss = root ? adoptCompiledCss(root, compiled.css) : () => {}

  return {
    js: compiled.js,
    css: compiled.css,
    mount: {
      dispose: () => {
        try {
          cleanup()
        } catch {
          /* cleanup is best-effort */
        }
        disposeCss()
        for (const n of Array.from(stageEl.childNodes)) {
          if (!before.has(n)) n.parentNode?.removeChild(n)
        }
      },
    },
  }
}
