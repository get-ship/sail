// Component library reference. Registers the primitive pack HERE (this route's
// graph — so the registration isn't tree-shaken and only loads on this page),
// then the page renders each <sui-*> live. `code` holds the usage snippets shown
// beside each one (kept in ./snippets so the scope scan stays clean).
import '@ship/sui-primitives' // side-effect: customElements.define for every <sui-*>
import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { code } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    return { code }
  },
})
