// Bundle entry for the BEHAVIOR harness: exposes the L1 `use:` actions in-page
// so the harness can attach them to the lowered markup and drive real
// interactions (click / keyboard) through Playwright.
import * as actions from '../runtime/src/actions/index'
// The boundary's contract is a RUNTIME one: a throw anywhere in the protected
// subtree is routed by `runProtected`, which walks up the DOM from the fault site
// looking for a registered handle. Asserting that from static markup would only
// re-test the CSS gate; to prove the routing, the spec has to throw for real.
import * as boundary from '../runtime/src/error-boundary'
// Motion's contract is COMPUTED PIXELS over time. The spec reproduces the
// compiler's exact construction order — `el.animate(...)` baked on a
// <template> clone (owned by the inert contents document) BEFORE insertion —
// because that ordering is where a whole bug class lives: an Animation minted
// there is bound to a timeline that never advances. `runLeave` is the seam
// `if.ts`/`for.ts` use to hold departing nodes until their exit settles.
import { __sail_motion_attach } from '../runtime/src/motion'
import { runLeave } from '../runtime/src/transition-runner'

const motion = { attach: __sail_motion_attach, runLeave }

;(globalThis as unknown as { SailActions: typeof actions }).SailActions = actions
;(globalThis as unknown as { SailBoundary: typeof boundary }).SailBoundary = boundary
;(globalThis as unknown as { SailMotion: typeof motion }).SailMotion = motion
