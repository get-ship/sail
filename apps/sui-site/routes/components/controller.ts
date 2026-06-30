// /components — the single component-library page. Registers the catalog +
// primitive packs so the catalog's preview slots find a defined tag for every
// component currently in the site bundle.
import '@ship/sui-primitives' // side-effect: customElements.define for every <sui-*>
import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller({
  scope: () => ({}),
})
