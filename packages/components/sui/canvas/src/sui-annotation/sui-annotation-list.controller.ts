import { controller, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-annotation-list.sui'
import type { AnnotationEvent } from './sui-annotation.internals'

export default controller({
  scope: (_ctx: ViewScope<Scope>, host?: Element) => {
    const staged = signal<AnnotationEvent[]>([])

    const emitRemove = (id: string): void => {
      host!.dispatchEvent(
        new CustomEvent('sui-annotation-remove', {
          detail: { id },
          bubbles: true,
          composed: true,
        }),
      )
    }

    return { staged, emitRemove }
  },

  mount(state, host) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const initial = (host as any).staged
    if (initial != null) state.staged = Array.isArray(initial) ? initial : []

    Object.defineProperty(host, 'staged', {
      configurable: true,
      get: () => state.staged,
      set: (v: AnnotationEvent[]) => { state.staged = Array.isArray(v) ? v : [] },
    })
  },
})
