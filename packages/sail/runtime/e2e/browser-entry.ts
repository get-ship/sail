// Browser bundle entry for the timeline e2e proof. `bun build` turns this
// into one self-contained script the Playwright spec injects into a page, so
// the code running in Chromium is the REAL runtime source — not a
// reimplementation of it, and not a mock.

import { signal, effect } from '../src/signal'
import { timeline } from '../src/timeline'
import { clock, springValue } from '../src/drivers'

declare global {
  interface Window {
    sail: {
      signal: typeof signal
      effect: typeof effect
      timeline: typeof timeline
      clock: typeof clock
      springValue: typeof springValue
    }
  }
}

window.sail = { signal, effect, timeline, clock, springValue }
