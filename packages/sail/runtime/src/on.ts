// Event handler wiring with modifier support.
//
// The sail compiler turns `on:click|preventDefault={fn}` into a call
// to this `on()` helper at hydrate time. Authors don't import this
// directly — they write the declarative form in templates.
//

import type { Disposer } from './bind'

/** Options for `on` — modifiers + standard `addEventListener` flags. */
export interface OnOptions {
  /** Modifier list, applied in declaration order. */
  modifiers?: Modifier[]
  /** Use the capture phase (`addEventListener` capture flag). */
  capture?: boolean
  /** Mark the listener as passive (cannot call `preventDefault`). */
  passive?: boolean
  /** Auto-remove the listener after the first dispatch. */
  once?: boolean
}

/**
 * One pre-handler effect: filter, transform, or rate-limit. Modifiers
 * compose in order; the parsed list is what `on` receives via
 * `OnOptions.modifiers`.
 */
export interface Modifier {
  kind:
    | 'preventDefault'
    | 'stopPropagation'
    | 'stopImmediatePropagation'
    | 'self'
    | 'trusted'
    | 'key'
    | 'debounce'
    | 'throttle'
  arg?: string
}

/**
 * Attach an event listener with Sail's modifier semantics.
 *
 * Modifier semantics — applied in the order listed in `opts.modifiers`:
 * - `preventDefault` — calls `event.preventDefault()` before the handler
 * - `stopPropagation` — calls `event.stopPropagation()` before the handler
 * - `stopImmediatePropagation` — same
 * - `self` — only fires when `event.target === el`
 * - `trusted` — only fires for user-initiated (`event.isTrusted`) events
 * - `key:NAME` — only fires when `matchesKey(event, NAME)` returns true
 *   (e.g. `key:Enter`, `key:Escape`, compounds like `key:meta+j`)
 * - `debounce:MS` — coalesces rapid fires; runs once `MS` ms after the
 *   last call
 * - `throttle:MS` — rate-limits to at most one fire per `MS` ms
 *
 * Compiler target: `on:click|preventDefault={fn}` template syntax.
 *
 * @param el Element (or any `EventTarget`) to attach to.
 * @param event Event name (`'click'`, `'keydown'`, ...).
 * @param handler Listener invoked once all modifier filters pass.
 * @param opts Modifiers and `addEventListener` flags.
 * @returns Disposer that removes the listener and cancels any pending
 *   debounce/throttle timers.
 *
 * @example
 * ```ts
 * const dispose = on(input, 'keydown', submit, {
 *   modifiers: [
 *     { kind: 'key', arg: 'Enter' },
 *     { kind: 'preventDefault' },
 *   ],
 * })
 * ```
 *
 * @see matchesKey for `key:` modifier semantics.
 */
export function on(
  el: EventTarget,
  event: string,
  handler: (event: Event) => unknown,
  opts: OnOptions = {},
): Disposer {
  const modifiers = opts.modifiers ?? []
  let timer: ReturnType<typeof setTimeout> | undefined
  let throttleArmed = true

  const wrapped = (e: Event) => {
    // Pre-handler modifiers
    for (const m of modifiers) {
      switch (m.kind) {
        case 'preventDefault':
          e.preventDefault()
          break
        case 'stopPropagation':
          e.stopPropagation()
          break
        case 'stopImmediatePropagation':
          e.stopImmediatePropagation()
          break
        case 'self':
          if (e.target !== el) return
          break
        case 'trusted':
          if (!e.isTrusted) return
          break
        case 'key': {
          if (!matchesKey(e as KeyboardEvent, m.arg ?? '')) return
          break
        }
      }
    }

    // Debounce / throttle take priority over the actual call
    const debounceMod = modifiers.find((m) => m.kind === 'debounce')
    const throttleMod = modifiers.find((m) => m.kind === 'throttle')

    if (debounceMod) {
      const ms = Number(debounceMod.arg) || 0
      if (timer !== undefined) clearTimeout(timer)
      timer = setTimeout(() => {
        timer = undefined
        handler(e)
      }, ms)
      return
    }

    if (throttleMod) {
      const ms = Number(throttleMod.arg) || 0
      if (!throttleArmed) return
      throttleArmed = false
      handler(e)
      timer = setTimeout(() => {
        throttleArmed = true
      }, ms)
      return
    }

    handler(e)
  }

  const listenerOpts: AddEventListenerOptions = {}
  if (opts.capture) listenerOpts.capture = true
  if (opts.passive) listenerOpts.passive = true
  if (opts.once) listenerOpts.once = true

  el.addEventListener(event, wrapped, listenerOpts)
  return () => {
    el.removeEventListener(event, wrapped, listenerOpts)
    if (timer !== undefined) clearTimeout(timer)
  }
}

/**
 * Match a keyboard event against a `key:` modifier spec. Used inside
 * `on` to gate `keydown`/`keyup` handlers; exposed for components that
 * need the same matching semantics outside of declarative bindings.
 *
 * - Single key: `Enter`, `Escape`, `a`
 * - Compound (modifier+key): `meta+j`, `ctrl+shift+a`, `alt+x`
 * - Aliases: `cmd` ≡ `meta`; `mod` ≡ `meta` on mac else `ctrl`
 *
 * Comparison is case-insensitive on the key name, but Shift state is
 * still required if `shift` is in the spec.
 *
 * @param e KeyboardEvent to test.
 * @param spec Lowercase or mixed-case key spec, modifiers separated by `+`.
 * @returns True when every required modifier and the final key match.
 *
 * @example
 * ```ts
 * if (matchesKey(e, 'mod+k')) openCommandPalette()
 * ```
 *
 * @see on for the declarative `key:` modifier.
 */
export function matchesKey(e: KeyboardEvent, spec: string): boolean {
  const parts = spec.split('+').map((p) => p.toLowerCase())
  const key = parts.pop() ?? ''
  const wantMeta = parts.includes('meta') || parts.includes('cmd')
  const wantCtrl = parts.includes('ctrl') || parts.includes('control')
  const wantShift = parts.includes('shift')
  const wantAlt = parts.includes('alt') || parts.includes('option')
  // `mod` = meta on darwin, ctrl elsewhere
  const wantMod = parts.includes('mod')
  const isMac =
    typeof navigator !== 'undefined' &&
    /Mac|iPhone|iPad|iPod/i.test(navigator.platform || navigator.userAgent || '')

  if (wantMeta && !e.metaKey) return false
  if (wantCtrl && !e.ctrlKey) return false
  if (wantShift && !e.shiftKey) return false
  if (wantAlt && !e.altKey) return false
  if (wantMod && !(isMac ? e.metaKey : e.ctrlKey)) return false

  return e.key.toLowerCase() === key
}
