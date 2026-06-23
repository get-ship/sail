import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

// A view WITH a controller can't be served from source — its compiled module
// imports `./controller`, so deck must produce a Rolldown bundle. The signal's
// initial value lives here (JS), so SSR can't know it: the server renders an
// empty reactive slot + hydration anchor, and the client fills `0` on mount.
// (Contrast `views/hello`, whose `{1 + 1}` is a constant the SSR pass folds.)
export default controller<Scope>({
  scope: () => ({ count: signal(0) }),
})
