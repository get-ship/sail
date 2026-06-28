import { controller } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

// No controller state needed — the <data> directive owns the live
// signal entirely. This file exists because the harness expects a
// paired controller; it's a no-op.
export default controller<Scope>({
  scope: () => ({}),
})
