import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

// The signal's initial value lives in JS, so SSR can't fold it: the server
// renders an empty reactive slot + hydration anchor; the client fills `0` on
// mount. Clicking proves signal reactivity drove a DOM update through deck.
export default controller<Scope>({
  scope: () => ({ count: signal(0) }),
})
