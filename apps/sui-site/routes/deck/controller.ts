// /deck — MOCKUP of the deck section of the shared SUI+deck site. Loads the
// deck theme and switches <html data-theme> to "deck" while this route is
// mounted, so the section reads as a distinct product; restores "sui" on unmount.
import { controller, onCleanup } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import '../../src/deck-theme.css'

export default controller({
  scope: () => ({}),
  mount() {
    const root = document.documentElement
    root.setAttribute('data-theme', 'deck')
    onCleanup(() => root.setAttribute('data-theme', 'sui'))
  },
})
