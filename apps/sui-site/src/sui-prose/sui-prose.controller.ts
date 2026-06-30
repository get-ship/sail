// Mirrors <sui-codeblock>/<sui-markdown>: bind the (already-rendered, trusted)
// HTML into the content div via bindHtml so it re-renders if `html` changes.
import { controller, bindHtml } from '@ship/sui-runtime'
import type { Scope } from './sui-prose.sui'

export default controller<Scope>({
  mount(state) {
    const el = state.$signals.contentEl() as HTMLElement | null
    if (!el) return () => {}
    return bindHtml(el, () => state.$signals.html() ?? '')
  },
})
