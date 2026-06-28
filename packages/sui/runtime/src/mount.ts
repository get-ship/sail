// mount() — F1 unified entry point.
//
// Replaces attach() (legacy, in @ship/sui-dom) with a payload-driven
// flow. Hosts (shell, view-host static, embed surfaces, tests) pass
// either a fetched ViewPayload or a name string; mount injects the
// SSR HTML, builds signals from payload.scope, imports the compiled
// view module, runs setup() + hydrate() + lifecycle.mount, opens any
// live SSE channel, and returns a typed MountHandle.
//
// Spec: .artifacts/design-f1-mount-unification.md
//
// IMPORTANT: this module owns the runtime-side half of the SUI mount
// contract. Anything that knows about HTML structure (importmap, boot
// scripts, doctype) lives OUTSIDE this file — in the host page.
//
// FILE_CAP_OVERRIDE: one cohesive concern — the mount() lifecycle. The private
// helpers (build scope from payload, resolve/load the view + islands, open the
// live SSE channel, extract lifecycle) are sequential steps of that one flow;
// they share the payload/scope context and have no use apart from mount().

import { signal, isSignal, type Signal } from './signal'
import type { RouteSignals } from './route-signals'
import { createScope, withScope } from './effect-scope'
import { morph } from './morph'
import type { LiveState, LiveSignal } from './live'
import type { ViewPayload as GeneratedViewPayload } from './generated'

/**
 * Wire shape returned by `GET /_deck/views/<name>[/<sub>]`. The base type
 * is generated from `crates/deck/deck-host/src/view_payload.rs::ViewPayload`
 * via `cargo xtask gen-types`. The local generic specializes `scope` —
 * Rust ships it as untyped JSON; per-view modules narrow it here.
 */
export type ViewPayload<Scope = Record<string, unknown>> =
  Omit<GeneratedViewPayload, 'scope'> & { scope: Scope }

/**
 * Host-provided loader. The runtime is host-agnostic — it does NOT know the
 * URL scheme for fetching view payloads or component bundles. The host (deck
 * view-host, shell) injects this adapter so the `/_deck/...` (or any) scheme
 * lives at the boundary, not in the framework.
 */
export interface SuiLoader {
  /** Resolve a view payload by `name[/sub]` (for the `mount(target, name)` form). */
  fetchView(nameOrPath: string): Promise<ViewPayload>
}

export interface MountOptions {
  /** Legacy single-client alias. Mirrored into `clients.ship` so views
   *  compiled either side of the `<client>` directive resolve the same
   *  object; prefer `clients: { ship }` going forward. */
  ship?: unknown
  /** Host-injected named data/ops clients, spread into scope so a view's
   *  `<data source="X.op()">` + handlers resolve bare `X` (the `<client>`-
   *  declared map). The implementor-named replacement for the single `ship`;
   *  `ship` itself is just a key here (aliased from `opts.ship` when only
   *  the legacy option is passed). */
  clients?: Record<string, unknown>
  route?: RouteSignals
  navigate?: (path: string) => void
  /** Host loader for by-name payload fetch + component-island loading. */
  loader?: SuiLoader
  /** HMR — overlay these values onto fresh signals after re-mount. */
  restoreScope?: Record<string, unknown>
  /** Where to inject the payload's css. Default: a `<style>` adjacent
   *  to `target`. `'none'` skips injection (host owns css). */
  cssScope?: 'adjacent' | 'document-head' | 'none'
  /** Pre-resolved compiled module — skips `import(payload.moduleUrl)`.
   *  Useful for tests, prefetching, and code-splitting decisions where
   *  the host already has the module in hand. */
  module?: { default: { setup: (ctx: unknown) => Record<string, unknown> } }
}

export interface MountHandle<Scope = Record<string, unknown>> {
  /** Tear down: invoke unmount, run cleanups, close SSE, remove style el. */
  dispose(): void
  /** Live signal map (writable per declared `<state>` keys). F2 types this. */
  scope: Scope
  /** The element passed to mount(). DOM stays in place after dispose. */
  element: HTMLElement
  /** HMR: dispose, reuse target, mount the new payload. Preserves
   *  signal values for keys present in both payloads' scope. */
  reload(next: ViewPayload): Promise<void>
}

interface SetupContext {
  ship?: unknown
  clients?: Record<string, unknown> | undefined
  route?: RouteSignals | undefined
  navigate?: ((path: string) => void) | undefined
  scope?: Record<string, unknown>
  stateKeys?: ReadonlySet<string>
}

interface CompiledModule {
  default: { setup: (ctx: SetupContext) => Record<string, unknown> }
}

interface Lifecycle {
  state: Record<string, unknown>
  mount?: (state: Record<string, unknown>, el: HTMLElement) => void
  unmount?: (state: Record<string, unknown>) => void
}
const LIFECYCLE_KEY = '__shipSetLifecycle'

/**
 * Mount a view into a DOM target.
 *
 * @param target Element to receive the rendered view. If the
 *               target already contains markup (SSR pre-render),
 *               mount morph-patches it toward payload.html, preserving
 *               node identity. Otherwise children are built fresh.
 * @param payloadOrName  A fetched ViewPayload, or a `name[/sub]`
 *               string the runtime fetches from `/_deck/views/...`.
 * @param opts   ship / route / navigate / restoreScope / cssScope.
 * @returns      MountHandle — call `dispose()` when done.
 */
export async function mount<Scope = Record<string, unknown>>(
  target: HTMLElement,
  payloadOrName: ViewPayload<Scope> | string,
  opts: MountOptions = {},
): Promise<MountHandle<Scope>> {
  if (!target) throw new Error('[sui/mount] target is required')

  const payload =
    typeof payloadOrName === 'string'
      ? await resolveView(payloadOrName, opts.loader)
      : payloadOrName

  // 0a. Seed route params from the payload. The daemon captured these
  //     from `[param]` directory segments during resolution, so a
  //     dynamic-route controller (`route.params.id()`) reads the real
  //     segment value without depending on client-side path matching.
  seedRouteParams(opts.route, payload.params)

  // 1. Adopt SSR markup if present (morph in place), else clean-mount.
  //    The view-host shell ships SSR content inside `target` before this
  //    runs; overwriting via `innerHTML = payload.html` would discard it
  //    and cause a 140-200ms blank flash. Morph reconciles in place and
  //    preserves focus/scroll/selection/custom-element state.
  if (hasHydratableMarkup(target)) {
    morph(target, payload.html)
  } else {
    const tmpl = (target.ownerDocument ?? document).createElement('template')
    tmpl.innerHTML = payload.html
    target.replaceChildren(...Array.from(tmpl.content.childNodes))
  }
  // 2. Inject CSS.
  const styleEl = injectCss(target, payload.css, opts.cssScope ?? 'adjacent')

  // 3. Build signals from payload.scope.
  const built = buildScopeFromPayload(payload.scope as Record<string, unknown>, opts.restoreScope)

  // 4. Import the compiled module (or use pre-resolved one).
  const mod = (opts.module ?? (await import(/* @vite-ignore */ payload.moduleUrl))) as CompiledModule
  if (!mod?.default || typeof mod.default.setup !== 'function') {
    throw new Error(
      `[sui/mount] module ${payload.moduleUrl} missing default.setup()`,
    )
  }

  // 5. Build setup context + run setup(ctx).
  // Route signals come from opts.route (host-provided) or the compiled
  // view's own import of @ship/sui-runtime/route. Mount no longer creates
  // a fallback instance — route.ts is out of the static bundle.
  //
  // Clients ↔ ship aliasing: the keyed `clients` map is the injection
  // surface; `ship` is one key on it plus the legacy ctx alias. A host
  // passing only `opts.ship` still populates `clients.ship`, and a host
  // passing only `clients: { ship }` still feeds `ctx.ship` — both styles
  // reach views compiled before AND after the `<client>` directive.
  // An explicit `opts.clients.ship` wins over `opts.ship` in the map.
  const ship = opts.ship ?? opts.clients?.['ship']
  const clients: Record<string, unknown> | undefined =
    opts.clients !== undefined || opts.ship !== undefined
      ? { ...(ship !== undefined ? { ship } : {}), ...opts.clients }
      : undefined
  warnUndeclaredClients(mod, payload.name, clients)
  const route = opts.route
  const navigate = opts.navigate ?? noop
  const baseScope: Record<string, unknown> = {
    ship,
    clients,
    // Spread each declared client by name so `<data source="X.op()">` resolves
    // bare `X`. Before `...built.scope` so author scope still shadows.
    ...(clients ?? {}),
    route,
    navigate,
    ...built.scope,
  }
  const setupCtx: SetupContext = {
    ship,
    clients,
    route,
    navigate,
    scope: { ...baseScope },
    stateKeys: built.stateKeys,
  }
  // Run setup() inside an effect scope so any `live()` / `data()` /
  // hook subscriptions allocated during setup register their
  // cleanup with this mount. The scope is disposed by `dispose()`
  // below — without it, EventSource subscriptions opened by
  // `<data live>` views leaked across every navigation and tabs
  // eventually OOM'd.
  const scope = createScope()
  const setupRet = withScope(scope, () =>
    mod.default.setup(setupCtx) as Record<string, unknown> | undefined,
  )

  // 6. setup()'s return shadows scope keys with duplicate signals; the
  //    SSR-built signals are the source of truth (writable proxy targets
  //    init.scope, SSE patches them). Restore them.
  const finalScope: Record<string, unknown> = { ...baseScope }
  if (setupRet) Object.assign(finalScope, setupRet)
  for (const k of built.stateKeys) {
    if (built.scope[k]) finalScope[k] = built.scope[k]
  }
  for (const k of built.liveKeys) {
    if (built.scope[k]) finalScope[k] = built.scope[k]
  }

  // 7. CSR path: if the target is still empty (no SSR HTML and no
  // `payload.html`), call the view's `__mount(target)` to create
  // the DOM. The compiled view's `__mount` is the CSR equivalent
  // of the SSR template — it builds the elements and wires the
  // text-open / attr-marker hooks that `hydrate` then activates.
  // (The legacy `attach()` path in @ship/sui-dom followed this same
  // shape; the new mount keeps it for the bundled-per-app world.)
  const clientMount = setupRet?.['__mount']
  if (
    typeof clientMount === 'function' &&
    target.childNodes.length === 0
  ) {
    try {
      clientMount(target)
    } catch (err) {
      console.error('[sui/mount] __mount threw:', err)
    }
  }

  // 8. Run hydrate(root, ctx) if the compiled module exposed it.
  const cleanups: Array<() => void> = []
  // Hoisted ahead of handle declaration so the async-mount branch
  // can guard against pushing a cleanup into a mount that already
  // disposed before its async mount() resolved.
  let disposed = false
  const hydrate = setupRet?.['hydrate']
  if (typeof hydrate === 'function') {
    try {
      const dispose = (hydrate as (r: HTMLElement, c: SetupContext) => unknown)(
        target,
        setupCtx,
      )
      if (typeof dispose === 'function') cleanups.push(dispose as () => void)
    } catch (err) {
      console.error('[sui/mount] hydrate threw:', err)
    }
  }

  // 8. Lifecycle mount.
  const lifecycle = extractLifecycle(setupRet)
  if (lifecycle?.mount) {
    // Run the controller's mount() INSIDE the same effect scope as setup (line
    // 206), exactly as compose.ts does for component mount. setup's withScope has
    // already closed by here, so a live()/data()/hook subscription opened in a
    // controller's mount() callback hit onCleanup() with CURRENT_SCOPE=null — a
    // no-op — and leaked one open SSE subscription per navigation (the OOM).
    const lifecycleMount = lifecycle.mount
    try {
      const result = withScope(scope, () => lifecycleMount(lifecycle.state, target)) as unknown
      // Controller mount may return any of:
      //   - void                — nothing to clean up
      //   - () => void          — disposer; idiomatic Solid/Svelte shape
      //   - Promise<void | () => void> — async mount with optional disposer
      //
      // The pre-fix branch only checked for Promise and surfaced
      // rejections, silently DROPPING the disposer return. 61 of 69
      // production controllers rely on `return () => …` for cleanup
      // (every MutationObserver, ResizeObserver, setInterval,
      // addEventListener tied to a host), so every navigation away
      // from a mounted view leaked listeners + observers + timers.
      // Honor the disposer in both sync and async cases.
      if (typeof result === 'function') {
        cleanups.push(result as () => void)
      } else if (
        result &&
        typeof result === 'object' &&
        typeof (result as Promise<unknown>).then === 'function'
      ) {
        ;(result as Promise<unknown>).then(
          (asyncResult) => {
            if (typeof asyncResult !== 'function') return
            const disposer = asyncResult as () => void
            if (disposed) {
              // The view was already disposed before the async mount
              // resolved — run the disposer immediately so its
              // resources are released, rather than letting it leak.
              try { disposer() } catch (err) {
                console.error('[sui/mount] async cleanup threw:', err)
              }
            } else {
              cleanups.push(disposer)
            }
          },
          (err) => {
            console.error('[sui/mount] controller mount() promise rejected:', err)
          },
        )
      }
    } catch (err) {
      console.error('[sui/mount] controller mount() threw:', err)
    }
  }

  // 9. Open live SSE channel if the payload declared one.
  //
  // SSE frames must patch the signals the *template* reads. The compiled
  // view (see crates/sui/compiler/src/emit/js/setup.rs::emit_data_decls)
  // emits `$scope.<name> = live(() => …)` for every `<data live>` and
  // returns those signals flattened into `setupRet` via
  // `Object.assign({}, $scope, { __mount, hydrate })`. That `$scope.<name>`
  // is the signal `bindWhileLoading` / `bindIf` close over; the mount-built
  // copy in `built.scope.<name>` is a server-rendered seed nobody reads.
  // Patching the seed leaves the template stuck on `loading` while the
  // view's signal idles in `error` (view-host: `ship` is undefined, the
  // live thunk throws synchronously) or in whatever state the in-view SDK
  // subscription happens to drive (shell). Target the view's signals so
  // the patch-signals channel resolves the template's loading branches.
  const subscribeUrl = pluckSubscribe(payload.scope as Record<string, unknown>)
  let liveSource: EventSource | null = null
  if (subscribeUrl && built.liveKeys.size > 0) {
    const liveScope = pickLiveSignals(
      setupRet,
      built.scope as Record<string, Signal<unknown>>,
      built.liveKeys,
    )
    liveSource = openLiveChannel(
      subscribeUrl,
      liveScope,
      built.liveKeys,
      target,
    )
    cleanups.push(() => liveSource?.close())
    // Gate the mount handle on the SSE channel reaching OPEN. Hosts and
    // e2e probes treat `__suiDispose` as the "channel is live" signal —
    // if a patch-elements POST hits the daemon before its per-view
    // broadcast registry has registered this connection's subscriber
    // (`apps/shipd/src/views/sui_patch_channel.rs::publish` does not
    // retain frames for late subscribers), the frame is silently
    // dropped. Capped at 3s so a wedged daemon path (the SSE handler
    // can stall before flushing headers if a live-data initial resolve
    // hangs) doesn't stall mount indefinitely.
    await awaitChannelOpen(liveSource, 3000)
  }

  // 10. Build the handle.
  // The handle's `scope` is the union of SSR-built signals (writable
  // patch targets) + the setup return's signals (any authored signals
  // the controller exposes). The runtime tests depend on `name`-style
  // signals being reachable through `handle.scope` so post-hydrate
  // writes can drive the binding's effect; without the union, the
  // handle exposes only the (empty) payload-derived scope and the
  // test's `sig.set(...)` becomes a no-op.
  const handleScope = Object.assign(
    Object.create(null),
    built.scope,
    setupRet ?? {},
  ) as Record<string, unknown> as unknown as Scope
  const handle: MountHandle<Scope> = {
    element: target,
    scope: handleScope,
    dispose() {
      if (disposed) return
      disposed = true
      if (lifecycle?.unmount) {
        try {
          lifecycle.unmount(lifecycle.state)
        } catch (err) {
          console.error('[sui/mount] controller unmount() threw:', err)
        }
      }
      for (const fn of cleanups) {
        try {
          fn()
        } catch (err) {
          console.error('[sui/mount] cleanup threw:', err)
        }
      }
      // Dispose the scope: closes any EventSource subscription opened
      // by `<data live>` / `live()` during setup, plus any other
      // resource registered via `onCleanup`.
      scope.dispose()
      styleEl?.remove()
    },
    async reload(next: ViewPayload) {
      // Snapshot current signal values for the keys present in `next.scope`.
      const restored: Record<string, unknown> = {}
      const nextScope = (next.scope ?? {}) as Record<string, unknown>
      for (const k of Object.keys(nextScope)) {
        const sig = (built.scope as Record<string, Signal<unknown>>)[k]
        if (sig) restored[k] = sig()
      }
      handle.dispose()
      const remounted = (await mount(target, next as ViewPayload, {
        ...opts,
        restoreScope: restored,
      })) as MountHandle<Scope>
      // Adopt the new mount's internals into this handle so the caller's
      // reference stays stable across reloads.
      handle.scope = remounted.scope
      handle.dispose = remounted.dispose
      handle.reload = remounted.reload
    },
  }
  return handle
}

/**
 * Compiled view modules carry their declared `<client>` names on the
 * default export (`clients: ["api", …]` — the mount-contract half emitted
 * by `emit/js/view.rs`). When the host's map is missing a declared key the
 * view's `<data source>` thunks fail opaquely later, so name the gap now.
 */
function warnUndeclaredClients(
  mod: CompiledModule,
  viewName: string,
  clients: Record<string, unknown> | undefined,
): void {
  const declared = (mod.default as { clients?: unknown }).clients
  if (!Array.isArray(declared)) return
  for (const name of declared) {
    if (typeof name !== 'string') continue
    if (clients === undefined || !(name in clients)) {
      console.warn(
        `[sui/mount] view "${viewName}" declares <client name="${name}"> but the host did not supply opts.clients.${name}`,
      )
    }
  }
}

function hasHydratableMarkup(el: HTMLElement): boolean {
  if (el.children.length > 0) return true
  for (const n of Array.from(el.childNodes)) {
    if (n.nodeType === Node.TEXT_NODE && (n.nodeValue ?? '').trim() !== '') {
      return true
    }
  }
  return false
}

function seedRouteParams(
  route: RouteSignals | undefined,
  params: Record<string, string | undefined> | undefined,
): void {
  if (!route || !params) return
  for (const [k, v] of Object.entries(params)) {
    if (typeof v !== 'string') continue
    // Reading the proxy key lazily creates the param signal; `.set`
    // overrides its URL-seeded value with the daemon-resolved one.
    route.params[k]?.set(v)
  }
}

async function resolveView(
  nameOrPath: string,
  loader: SuiLoader | undefined,
): Promise<ViewPayload> {
  if (!loader) {
    throw new Error(
      `[sui/mount] mounting by name ("${nameOrPath}") requires opts.loader — the ` +
        `host must provide a SuiLoader. The runtime does not know the view URL scheme.`,
    )
  }
  return loader.fetchView(nameOrPath)
}

interface BuiltScope {
  scope: Record<string, Signal<unknown>>
  stateKeys: Set<string>
  liveKeys: Set<string>
}

function buildScopeFromPayload(
  payloadScope: Record<string, unknown>,
  restore?: Record<string, unknown>,
): BuiltScope {
  const scope: Record<string, Signal<unknown>> = {}
  const stateKeys = new Set<string>()
  const liveKeys = new Set<string>()
  for (const [k, v] of Object.entries(payloadScope)) {
    if (k === '__subscribe__') continue
    if (isLiveStateEnvelope(v)) {
      const init = restore && k in restore ? restore[k] : v
      scope[k] = signal<LiveState<unknown>>(init as LiveState<unknown>) as unknown as LiveSignal<unknown> as unknown as Signal<unknown>
      liveKeys.add(k)
    } else {
      const init = restore && k in restore ? restore[k] : v
      scope[k] = signal<unknown>(init)
      stateKeys.add(k)
    }
  }
  return { scope, stateKeys, liveKeys }
}

function pluckSubscribe(payloadScope: Record<string, unknown>): string | undefined {
  const v = payloadScope['__subscribe__']
  return typeof v === 'string' ? v : undefined
}

/**
 * Resolve `EventSource.readyState === OPEN` (or fall back at `timeoutMs`).
 * Used to gate the mount handle on the live channel actually existing on
 * the daemon side — `EventSource` connects asynchronously, and the SSE
 * handler does not register its broadcast subscriber until the request
 * arrives, so a publish that beats the registration is dropped.
 */
function awaitChannelOpen(es: EventSource, timeoutMs: number): Promise<void> {
  // readyState: 0 CONNECTING, 1 OPEN, 2 CLOSED.
  if (es.readyState === 1) return Promise.resolve()
  return new Promise((resolve) => {
    let settled = false
    let timerId: ReturnType<typeof setTimeout> | undefined
    const settle = (): void => {
      if (settled) return
      settled = true
      if (timerId !== undefined) clearTimeout(timerId)
      es.removeEventListener('open', settle)
      es.removeEventListener('error', settle)
      resolve()
    }
    es.addEventListener('open', settle)
    // EventSource emits `error` on transport errors and also when the
    // browser closes the channel during reconnect — either way the
    // initial connect attempt has finished. Don't keep mount() waiting.
    es.addEventListener('error', settle)
    if (timeoutMs > 0) {
      // Store the id so settle() cancels it — else the timer holds its closure
      // (and the resolve continuation) alive for the full timeout after the
      // channel already opened/errored.
      timerId = setTimeout(settle, timeoutMs)
    }
  })
}

/**
 * Resolve which signals the SSE patch-signals channel should target for
 * each declared live key. Prefers the signal the compiled view's setup
 * exposed (the one the template's hydrate bindings actually read), and
 * falls back to the SSR-built seed when setup did not surface one — keeps
 * legacy / hand-rolled view modules that don't flatten `$scope` working.
 */
function pickLiveSignals(
  setupRet: Record<string, unknown> | undefined,
  fallback: Record<string, Signal<unknown>>,
  liveKeys: ReadonlySet<string>,
): Record<string, Signal<unknown>> {
  const out: Record<string, Signal<unknown>> = {}
  for (const k of liveKeys) {
    const fromSetup = setupRet?.[k]
    if (isSignal(fromSetup)) {
      out[k] = fromSetup as Signal<unknown>
    } else if (fallback[k]) {
      out[k] = fallback[k]!
    }
  }
  return out
}

function isLiveStateEnvelope(v: unknown): v is LiveState<unknown> {
  if (!v || typeof v !== 'object') return false
  const k = (v as { kind?: unknown }).kind
  return k === 'loading' || k === 'ok' || k === 'error'
}

function injectCss(
  target: HTMLElement,
  css: string,
  scope: Exclude<MountOptions['cssScope'], undefined>,
): HTMLStyleElement | null {
  if (!css || scope === 'none') return null
  const doc = target.ownerDocument ?? document
  const style = doc.createElement('style')
  style.setAttribute('data-sui-css', '1')
  style.textContent = css
  if (scope === 'document-head') {
    doc.head.appendChild(style)
  } else {
    // 'adjacent' — sibling before target so cascade order matches the
    // standalone (`<link>` in <head>) flow.
    target.parentNode?.insertBefore(style, target)
  }
  return style
}

function extractLifecycle(ret: unknown): Lifecycle | undefined {
  if (!ret || typeof ret !== 'object') return undefined
  const lc = (ret as Record<string, unknown>)[LIFECYCLE_KEY]
  if (lc && typeof lc === 'object') return lc as Lifecycle
  return undefined
}

function noop(_p: string): void {
  /* default navigate */
}

// ── Live SSE channel (parallel to ui-dom's attach/live-channel) ────────
//
// Mount owns its own copy because @ship/sui-runtime cannot import
// @ship/sui-dom (would invert the layering). Once F1.6 deletes attach(),
// the ui-dom copy goes too.

interface PatchSignalsPayload {
  name: string
  value: LiveState<unknown>
}

function openLiveChannel(
  url: string,
  scope: Record<string, Signal<unknown>>,
  liveKeys: Set<string>,
  root: HTMLElement,
): EventSource {
  const es = new EventSource(url)
  es.addEventListener('patch-signals', (event: MessageEvent) => {
    let payload: PatchSignalsPayload
    try {
      payload = JSON.parse(event.data as string) as PatchSignalsPayload
    } catch {
      console.warn('[sui/mount] patch-signals: bad payload', event.data)
      return
    }
    const { name, value } = payload
    if (!liveKeys.has(name)) return
    const sig = scope[name]
    if (!sig) return
    ;(sig as Signal<LiveState<unknown>>).set(value)
  })
  es.addEventListener('patch-elements', (event: MessageEvent) => {
    applyPatchElementsFrame(root, event.data as string)
  })
  es.addEventListener('error', () => {
    // Built-in EventSource reconnect handles transient drops.
  })
  return es
}

// ── patch-elements applier ────────────────────────────────────────────
//
// Out-of-band DOM patches pushed by the daemon's
// `/_deck/views/<name>/patch-elements` endpoint. The trust boundary is the
// SSE auth in shipd's `sui_agent_push.rs` — by the time a frame reaches
// here it's already a trusted caller. The runtime's only job is to scope
// the selector to the view root: `[data-sui-root][data-view="<name>"]`
// is the mount target, so anything outside is silently dropped (with a
// console warning) — patches can't escape the hosted view even if a
// publisher sends `body` or `html`.

interface PatchElementsPayload {
  selector: string
  mode: 'replace' | 'append' | 'prepend' | 'before' | 'after' | 'morph'
  html: string
  settle?: number
}

function applyPatchElementsFrame(root: HTMLElement, raw: string): void {
  let payload: PatchElementsPayload
  try {
    payload = JSON.parse(raw) as PatchElementsPayload
  } catch {
    console.warn('[sui/mount] patch-elements: malformed payload', raw)
    return
  }
  const { selector, mode, html } = payload
  if (typeof selector !== 'string' || selector.trim() === '') {
    console.warn('[sui/mount] patch-elements: empty selector')
    return
  }
  let target: Element | null = null
  try {
    target = root.querySelector(selector)
  } catch (err) {
    console.warn(
      `[sui/mount] patch-elements: invalid selector ${selector!}:`,
      err,
    )
    return
  }
  if (!target) {
    // Either no match anywhere, or the match lives outside the view
    // root. The view-host shell tagged the root with
    // `data-sui-root[data-view]` precisely so external publishers can't
    // mutate the rest of the page through this channel.
    console.warn(
      `[sui/mount] patch-elements: selector ${selector!} matched no element inside the view root; dropped`,
    )
    return
  }
  applyPatchElementsTarget(target, mode, html)
}

function applyPatchElementsTarget(
  target: Element,
  mode: PatchElementsPayload['mode'],
  html: string,
): void {
  const doc = target.ownerDocument ?? document
  switch (mode) {
    case 'replace':
      target.outerHTML = html
      return
    case 'append': {
      const tmpl = doc.createElement('template')
      tmpl.innerHTML = html
      target.append(...Array.from(tmpl.content.childNodes))
      return
    }
    case 'prepend': {
      const tmpl = doc.createElement('template')
      tmpl.innerHTML = html
      target.prepend(...Array.from(tmpl.content.childNodes))
      return
    }
    case 'before': {
      const tmpl = doc.createElement('template')
      tmpl.innerHTML = html
      target.before(...Array.from(tmpl.content.childNodes))
      return
    }
    case 'after': {
      const tmpl = doc.createElement('template')
      tmpl.innerHTML = html
      target.after(...Array.from(tmpl.content.childNodes))
      return
    }
    case 'morph': {
      // Parse the HTML as a single top-level element, then morph the
      // existing `target` toward it — preserves the node identity (and
      // focus/selection/CE state) when tags match. Falls back to
      // outerHTML replace when tags differ or no source element is
      // present.
      const tmpl = doc.createElement('template')
      tmpl.innerHTML = html
      const source = tmpl.content.firstElementChild
      if (!source) {
        console.warn('[sui/mount] patch-elements: morph with no source element')
        return
      }
      if (source.tagName !== target.tagName) {
        target.outerHTML = html
        return
      }
      morph(target, source)
      return
    }
    default:
      console.warn(`[sui/mount] patch-elements: unknown mode ${String(mode)}`)
  }
}
