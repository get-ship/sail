import { controller, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-annotation-list.sui'
import type { AnnotationEvent } from './sui-annotation.internals'

export default controller({
  scope: (_ctx: ViewScope<Scope>, _host?: Element) => {
    const staged = signal<AnnotationEvent[]>([])
    // The host element is not present in the scope factory (the post-
    // convergence "SUI names no host" compile stops passing it as the 2nd
    // arg), so read it through this signal — set in `mount` once the
    // runtime hands over the real element. The template wires
    // `on:click={() => emitRemove(a.id)}`, so the emit must read the host
    // from a signal, not the dead scope `host` arg.
    const hostEl = signal<HTMLElement | null>(null)

    const emitRemove = (id: string): void => {
      hostEl()?.dispatchEvent(
        new CustomEvent('sui-annotation-remove', {
          detail: { id },
          bubbles: true,
          composed: true,
        }),
      )
    }

    return { staged, emitRemove, hostEl }
  },

  mount(state, host) {
    // The runtime hands the real host element here. The scope's `emitRemove`
    // closure reads through `hostEl` (the dead-`host` workaround) — publish
    // the element now that it exists, before the first user click.
    state.hostEl = host as HTMLElement
    // Initial `staged` value: the parent layer sets it as a JS property on
    // this element before mount (the host→list contract). Narrow the access
    // with a guard rather than `as any` — the property is the value type
    // we expect, not an untyped escape hatch.
    if ('staged' in host) {
      const v = host.staged
      if (Array.isArray(v)) state.staged = v as AnnotationEvent[]
    }

    // The parent layer mutates `staged` via the host JS property
    // (`list.staged = [...]`); install the accessor so it routes through
    // the signal.
    Object.defineProperty(host, 'staged', {
      configurable: true,
      get: () => state.staged,
      set: (v: AnnotationEvent[]) => { state.staged = Array.isArray(v) ? v : [] },
    })
  },
})
