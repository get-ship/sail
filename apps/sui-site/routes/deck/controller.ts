// /deck — the Deck overview. The shell owns the deck (violet) theme via
// data-theme; this controller only supplies the page's code snippets.
import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { quickstart, layers } from './snippets'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({ quickstart, layers }),
})
