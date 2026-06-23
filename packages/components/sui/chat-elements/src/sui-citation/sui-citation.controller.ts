import { controller, ref } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-citation.sui'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const pillEl = ref<HTMLButtonElement>(null)
    return { pillEl }
  },

  mount(state, host) {
    const btn = state.pillEl as HTMLButtonElement | null
    if (!btn) return

    const onClick = () => {
      host.dispatchEvent(
        new CustomEvent('sui-citation-open', {
          detail: { source: state.source, index: state.index },
          bubbles: true,
          composed: true,
        }),
      )
    }
    btn.addEventListener('click', onClick)
    return () => btn.removeEventListener('click', onClick)
  },
})
