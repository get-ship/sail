// Mirrors <sui-markdown>: bind the highlighted HTML into the <pre> via bindHtml,
// so it re-renders if `code` changes. highlight() escapes the source and only
// emits its own token spans.
import { controller, bindHtml } from '@ship/sui-runtime'
import type { Scope } from './sui-codeblock.sui'
import { highlight } from '../highlight'

export default controller<Scope>({
  mount(state) {
    const el = state.$signals.preEl() as HTMLElement | null
    if (!el) return () => {}
    return bindHtml(el, () => highlight(state.$signals.code() ?? ''))
  },
})
