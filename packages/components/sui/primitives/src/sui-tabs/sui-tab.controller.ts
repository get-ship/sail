import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-tab.sui'

export default controller<Scope>({
  mount(state, host) {

    // Sync tabIndex on the inner button based on the `active` prop.
    // sui-tabs sets the `active` attribute; this keeps tabIndex in sync
    // so only the active tab is reachable via Tab key (roving tabindex).
    const stopTabIndex = effect(() => {
      const btn = state.$signals.btn()
      if (!btn) return
      btn.tabIndex = state.$signals.active() ? 0 : -1
    })

    return () => {
      stopTabIndex()
    }
  },
})
