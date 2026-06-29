import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

// The signal's initial value lives in JS, so SSR can't fold it: the server
// renders the static structure + a hydration anchor for the count slot; the
// client fills `0` on hydrate. Clicking proves reactivity survived hydration.
export default controller<Scope>({
  scope: () => ({ count: signal(0) }),
})
