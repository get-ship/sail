// /components controller. Registers the catalog element + the primitive
// packs so the catalog's preview slots find a defined tag for every component
// in the site bundle.
import '../../src/register-all' // side-effect: register EVERY pack's <sui-*> so every catalog card finds a defined tag
import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller({
  scope: () => ({}),
})
