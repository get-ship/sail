// /deck — the Deck product section of the Ship UI site. Theme is owned by the
// shell (data-theme by product), so this route just renders content.
import { controller } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller({
  scope: () => ({}),
})
