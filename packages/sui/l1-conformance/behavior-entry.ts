// Bundle entry for the BEHAVIOR harness: exposes the L1 `use:` actions in-page
// so the harness can attach them to the lowered markup and drive real
// interactions (click / keyboard) through Playwright.
import * as actions from '../runtime/src/actions/index'

;(globalThis as unknown as { SuiActions: typeof actions }).SuiActions = actions
