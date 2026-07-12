// Bundle entry for the BEHAVIOR harness: exposes the L1 `use:` actions in-page
// so the harness can attach them to the lowered markup and drive real
// interactions (click / keyboard) through Playwright.
import * as actions from '../runtime/src/actions/index'
// The boundary's contract is a RUNTIME one: a throw anywhere in the protected
// subtree is routed by `runProtected`, which walks up the DOM from the fault site
// looking for a registered handle. Asserting that from static markup would only
// re-test the CSS gate; to prove the routing, the spec has to throw for real.
import * as boundary from '../runtime/src/error-boundary'

;(globalThis as unknown as { SuiActions: typeof actions }).SuiActions = actions
;(globalThis as unknown as { SuiBoundary: typeof boundary }).SuiBoundary = boundary
