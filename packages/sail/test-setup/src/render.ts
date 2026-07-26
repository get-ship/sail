// Public author-facing test API for SAIL routes and components.
//
// This is the "shared unlock": a `views/<name>/` or component author imports
// ONE function, points it at their `.sail` source + controller, and asserts
// against a real (jsdom) DOM tree — driven through the SAME production Rust/WASM
// compiler the daemon uses, so a passing test means the production emit shape
// renders and behaves.
//
// The compile + eval + mount mechanics live in `@ship/sail-dom/test-utils`
// (one source of truth, exercised by sail-dom's own suite). This module is the
// ergonomic surface on top: it returns the mounted root, the live scope, and
// helpers to fire events and flush microtasks.
//
//   import { renderRoute } from '@ship/test-setup'
//   import gitRoute from '../controller'
//   const TEMPLATE = readFileSync(new URL('./page.sail', import.meta.url), 'utf8')
//
//   const v = renderRoute(TEMPLATE, { controller: gitRoute, providers: { api } })
//   expect(v.html()).toContain('main')
//   v.fire('button.refresh', 'click')
//   await v.flush()
//   v.unmount()

import { compileSui, evalCompiledJs, evalInlineController } from '@ship/sail-dom/test-utils'
import { route as makeRoute } from '@ship/sail-runtime/route'
import type { ControllerExport } from '@ship/sail-runtime'

/** A controller's default export (the `controller(fn)` shape, which carries
 *  a `.setup`), a `ControllerExport<Scope>` with a typed setup, or a bare
 *  setup function. */
export type ControllerInput =
  | { setup: (ctx: unknown) => unknown }
  | ControllerExport<any>
  | ((ctx: unknown) => Record<string, unknown> | void)

export interface RenderOpts {
  /** The route's controller — its default export, or a raw setup function.
   *  Omit for templates with no controller. */
  controller?: ControllerInput
  /** Flat named providers the route reads (same surface as mount inject). */
  providers?: Record<string, unknown>
  /** Initial URL backing the `route` signal. Defaults to `http://localhost/`. */
  url?: string
  /** Navigation sink — called when the route navigates. Defaults to a no-op. */
  navigate?: (path: string) => void
  /** On-disk path of the `.sail` source. Pass it when the source was read off
   *  disk (`readFileSync(new URL('./page.sail', import.meta.url))`) so the
   *  compiler resolves the source's own styling policy and runs the four
   *  controller-aware passes — the harness then rejects exactly what the daemon
   *  + CLI do. Use `fileURLToPath(new URL('./page.sail', import.meta.url))`. */
  sourcePath?: string
}

export interface Rendered {
  /** The container the route mounted into. Query it with `.querySelector`. */
  root: HTMLElement
  /** The route's reactive scope (signals + derived + controller keys). */
  scope: Record<string, unknown>
  /** Current rendered HTML of the root. */
  html(): string
  /** Dispatch a bubbling DOM event at the first element matching `selector`.
   *  Throws if nothing matches — a missing target is a test bug, not a pass. */
  fire(selector: string, type: string, init?: EventInit): boolean
  /** Resolve after a microtask + macrotask so signal effects have flushed. */
  flush(): Promise<void>
  /** Unmount the route and remove the root from the document. */
  unmount(): void
}

// Any non-empty marker makes the compiler emit `import setup from "<marker>"`,
// which the eval layer rewrites to the injected controller. The exact value is
// irrelevant — the import-strip matches any setup import.
const SETUP_MARKER = '__test_setup__'

/**
 * Compile a `.sail` route from source, mount it against jsdom with the given
 * controller + fakes, and return handles to assert and drive it.
 */
export function renderRoute(source: string, opts: RenderOpts = {}): Rendered {
  const usesController = opts.controller != null
  const compiled = compileSui(source, {
    ...(usesController ? { setupMarker: SETUP_MARKER } : {}),
    ...(opts.sourcePath ? { sourcePath: opts.sourcePath } : {}),
  })
  const def = evalCompiledJs(compiled.js, opts.controller as never)

  const root = document.createElement('div')
  document.body.appendChild(root)

  const ctx = {
    ...(opts.providers ?? {}),
    ...(opts.providers ? { providers: opts.providers } : {}),
    // Base the URL so relative paths ("/authority") from given.url resolve.
    route: makeRoute({ url: new URL(opts.url ?? '/', 'http://localhost') }),
    navigate: opts.navigate ?? (() => {}),
  }
  const scope = def.setup(ctx) as Record<string, unknown> & {
    __mount: (c: HTMLElement) => () => void
  }
  const cleanup = scope.__mount(root)

  return {
    root,
    scope,
    html: () => root.innerHTML,
    fire(selector, type, init) {
      const el = root.querySelector(selector)
      if (!el) throw new Error(`renderRoute.fire: no element matches "${selector}"`)
      return el.dispatchEvent(new Event(type, { bubbles: true, ...init }))
    },
    flush: () => new Promise((r) => setTimeout(r, 0)),
    unmount() {
      try {
        cleanup()
      } catch {
        /* mount cleanup is best-effort */
      }
      root.remove()
    },
  }
}

export interface RenderComponentOpts {
  /** Attribute props to set on the element before it connects. */
  attrs?: Record<string, string | boolean>
  /** Override the controller the source declares — its default export, or a raw
   *  setup function. An SFC's `<controller>` block is resolved automatically, so
   *  pass this only to inject a fake, or when the block's imports need the real
   *  module graph (a third-party dep the harness cannot eval): import the virtual
   *  `<file>.sail.controller.ts` and hand it in. */
  controller?: ControllerInput
  /** On-disk path of the `.sail` source. Pass it when the source was read off
   *  disk so the compiler resolves the source's own styling policy and runs
   *  the four controller-aware passes (scope_bound, scope_member,
   *  bind_this_target, on_callable) — the harness then rejects exactly what the
   *  daemon + CLI do. Use
   *  `fileURLToPath(new URL('./foo.sail', import.meta.url))`. */
  sourcePath?: string
}

/**
 * Compile a component `.sail` (one declaring `<prop>`s) under the given tag,
 * register it, instantiate it with `attrs`, and return handles. Assert against
 * the host element (light DOM). `shadow` is usually null unless a residual
 * containment wrapper attached one — query via `el` / `root.querySelector(tag)`.
 */
export function renderComponent(
  source: string,
  tag: string,
  opts: RenderComponentOpts = {},
): Omit<Rendered, 'scope'> & { el: HTMLElement; shadow: ShadowRoot | null } {
  // Component compile is a side-effecting `component(tag, def)` that registers
  // the custom element. When a controller is supplied, compile with the setup
  // marker so eval rewrites the `import setup` line to the injected controller
  // (mirrors renderRoute); otherwise the template renders without scope.
  // An SFC carries its controller inline — the compile returns the body and
  // the emitted template imports it as `setup`. An explicit `opts.controller`
  // still wins, for a fake or a block whose imports need the real module graph.
  const probe = compileSui(source, {
    componentName: tag,
    mode: 'component',
    ...(opts.sourcePath ? { sourcePath: opts.sourcePath } : {}),
  })
  const inline = probe.controller
  const usesController = opts.controller != null || inline != null
  const compiled = usesController
    ? compileSui(source, {
        componentName: tag,
        mode: 'component',
        setupMarker: SETUP_MARKER,
        ...(opts.sourcePath ? { sourcePath: opts.sourcePath } : {}),
      })
    : probe
  const setup =
    opts.controller ?? (inline != null ? evalInlineController(inline) : undefined)
  evalCompiledJs(compiled.js, setup as never)

  const root = document.createElement('div')
  document.body.appendChild(root)
  const el = document.createElement(tag)
  for (const [k, v] of Object.entries(opts.attrs ?? {})) {
    if (v === false) continue
    el.setAttribute(k, v === true ? '' : v)
  }
  root.appendChild(el)

  return {
    root,
    el,
    shadow: el.shadowRoot,
    html: () => root.innerHTML,
    fire(selector, type, init) {
      const target = root.querySelector(selector)
      if (!target) throw new Error(`renderComponent.fire: no element matches "${selector}"`)
      return target.dispatchEvent(new Event(type, { bubbles: true, ...init }))
    },
    flush: () => new Promise((r) => setTimeout(r, 0)),
    unmount() {
      el.remove()
      root.remove()
    },
  }
}
