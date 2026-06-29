import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

// A controller signal — the HMR spec drives it to a non-default value, edits
// page.sui on disk, and asserts the value survives the soft reload.
export default controller<Scope>({
  scope: () => ({ count: signal(0) }),
})
