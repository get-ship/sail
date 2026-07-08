// <sui-nav-top> — top application bar. A pure layout primitive:
// the bar chrome (height, border, background) and three named
// slots (brand, default, actions). No state. The slotted
// children own their own state (whatever widgets the host
// chooses to mount — the nav-top itself is domain-neutral).
//
// The only behavior is firing `sui:nav-top:ready` once after
// first mount so the app can wire post-mount concerns.
import { controller } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-nav-top.sui'

export default controller<Scope>({
  mount(_state, host) {
    host.dispatchEvent(new CustomEvent('sui:nav-top:ready', { bubbles: true, composed: true }))
  },
})
