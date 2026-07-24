// Host-side primitives for component controllers.
//
// `mount()` controller bodies repeated six small patterns across 69
// production components — `dispatchEvent(new CustomEvent(...))`, host
// method exposure, raw `addEventListener` pairs, slotchange queries,
// and effect-driven `setAttribute` reflection. Each pattern is its
// own correctness foot-gun (forget `bubbles+composed` and the event
// dies at the shadow boundary; forget `removeEventListener` and the
// listener leaks across reconnects). These primitives drain the
// boilerplate AND auto-enroll cleanups via the active `effect-scope`
// so authors can't forget — `defineComponent` wraps the factory in
// `withScope`, so everything called from a controller `mount()` body
// has a scope to register against.
//
// See .artifacts/job-spec.md (component-primitives job).
//
// Forbidden in controller code: raw `addEventListener` /
// `dispatchEvent(new CustomEvent(...))` / imperative `setAttribute`
// effects when one of the primitives below fits. Use them instead.
//
// `bindHostAttr` reuses `bindAttr` from `./bind` — its present-vs-
// absent semantics already match what controllers want
// (`null/undefined/false/''` removes; `true` is presence-only).

import { bindAttr } from './bind'
import { onCleanup } from './effect-scope'
import { effect } from './signal'

// ─── Type augmentation: host.emit / host.expose ──────────────────────
//
// The methods are attached by `attachHostApi` (called from
// `defineComponent`) so every host element produced by the runtime
// carries them. The optional flag matches DOM tradition — spec-defined
// methods are always present on the prototype, but our host-API
// methods are only present after `connectedCallback` has run.
declare global {
  interface HTMLElement {
    /**
     * Dispatch a `CustomEvent` from this element. Defaults to
     * `bubbles: true, composed: true` — the right behavior for a
     * shadow-DOM custom element whose event should escape to the
     * host's light tree. Pass `detail` to attach a payload.
     */
    emit?<T = unknown>(name: string, detail?: T): void
    /**
     * Attach a bag of imperative methods to the host so consumers
     * can call `el.focus()`, `el.clear()`, etc. The methods are
     * removed from the host on disconnect (via the active
     * effect-scope), so a reconnect gets fresh bindings rather
     * than stale closures over the previous mount's signals.
     */
    expose?(methods: Record<string, (...args: never[]) => unknown>): void
  }
}

/** Dispatch a `CustomEvent` from `host` with `bubbles+composed`
 *  defaulted true. Free-function form of `host.emit` — useful when
 *  the host reference isn't typed as `HTMLElement` (e.g. `Element`
 *  from a generic ref). */
export function emit<T = unknown>(host: EventTarget, name: string, detail?: T): void {
  host.dispatchEvent(new CustomEvent(name, { detail, bubbles: true, composed: true }))
}

/** Attach `methods` to `host` and register a cleanup to delete them
 *  on the next scope disposal. Free-function form of `host.expose`. */
export function expose(
  host: HTMLElement,
  methods: Record<string, (...args: never[]) => unknown>,
): void {
  const target = host as unknown as Record<string, unknown>
  const names = Object.keys(methods)
  for (const name of names) target[name] = methods[name]
  onCleanup(() => {
    for (const name of names) {
      // Only delete if the property still points to the function we
      // installed. A re-mount that runs `expose` again may have
      // overwritten it with a fresh closure; deleting unconditionally
      // would null out the live one.
      if (target[name] === methods[name]) {
        try {
          delete target[name]
        } catch {
          // Some host elements (like form-associated subclasses) may
          // have the property as non-configurable inherited; assigning
          // undefined is the next best thing.
          target[name] = undefined
        }
      }
    }
  })
}

/**
 * Wire `host.emit` and `host.expose` onto a host element. Idempotent —
 * called from `defineComponent`'s `connectedCallback` once per connect.
 *
 * `emit` is bound to the host so callers can do
 * `host.emit('sail-foo', detail)` without any imports. `expose` is a
 * thin convenience that calls the free-function `expose` against
 * `host`; it exists so authors don't have to thread the host argument
 * through every controller body.
 */
export function attachHostApi(host: HTMLElement): void {
  const target = host as HTMLElement & {
    emit: <T>(name: string, detail?: T) => void
    expose: (methods: Record<string, (...args: never[]) => unknown>) => void
  }
  target.emit = function <T>(name: string, detail?: T): void {
    emit(host, name, detail)
  }
  target.expose = function (methods): void {
    expose(host, methods)
  }
}

/**
 * Add an event listener to `target` and register the matching
 * removeEventListener as a cleanup with the active effect-scope.
 *
 * Replaces the manual `addEventListener` / `removeEventListener`
 * pair that controllers were balancing by hand — 140 add / 129
 * remove call sites pre-migration; the gap was the leak surface.
 *
 * @example
 * ```ts
 * bindNativeEvent(scope.inputEl()!, 'input', (e) => host.emit('sail-input', { value: (e.target as HTMLInputElement).value }))
 * ```
 */
export function bindNativeEvent(
  target: EventTarget,
  event: string,
  handler: (event: Event) => void,
  opts?: AddEventListenerOptions | boolean,
): void {
  target.addEventListener(event, handler, opts)
  onCleanup(() => target.removeEventListener(event, handler, opts))
}

/**
 * Subscribe to a native `<slot>`'s assigned-elements list (shadow slot API).
 * Residual helper for opt-in foreign-content shadow wrappers — Sail L1 uses
 * light-DOM projection (`projectSlots`), not `assignedElements()`.
 *
 * `render` runs once on initial mount AND on every `slotchange`.
 *
 * @param slot Reactive read of the slot element (typically
 *   `() => scope.fooSlot()`). May return `null` on first run if the
 *   slot is conditionally rendered.
 * @param render Callback invoked with the assigned elements. Called
 *   once on initial subscribe, then on every slotchange.
 */
export function slotted(
  slot: () => HTMLSlotElement | null,
  render: (els: Element[]) => void,
): void {
  const stop = effect(() => {
    const s = slot()
    if (!s) return
    const update = (): void => {
      render(s.assignedElements())
    }
    s.addEventListener('slotchange', update)
    update()
    return () => s.removeEventListener('slotchange', update)
  })
  onCleanup(stop)
}

/**
 * Reactive reflect of a value to a host attribute. Returning `null`,
 * `undefined`, `false`, or `''` from `get` removes the attribute;
 * `true` writes a presence-only attribute; anything else is
 * stringified and assigned. Cleanup auto-registers with the active
 * effect-scope.
 *
 * Templates use `bindAttr` directly via the `attr={expr}` compiler
 * target. `bindHostAttr` is the controller-side equivalent — same
 * underlying machinery, but the cleanup registers with the scope so
 * authors don't have to capture the disposer.
 */
export function bindHostAttr(host: Element, name: string, get: () => unknown): void {
  onCleanup(bindAttr(host, name, get))
}
