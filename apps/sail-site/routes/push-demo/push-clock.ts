// push-clock — a REAL push source fixture for the browser proof of
// query()'s PUSH path (data-landing-spec.md, "BROWSER" gate). `feed` is an
// ES-imported client whose `clock.stream()` method returns a
// `LiveSource<Tick>` (a `{ subscribe }` shape) rather than a Promise — so
// query() routes it to the cache's push path and the entry is fed by the
// interval subscription, one `ok` per tick, until the view is disposed.

import type { LiveSource, LiveSignal } from '@ship/sail-runtime'

export interface Tick {
  n: number
  at: string
}

export const feed = {
  clock: {
    // Emits a tick every 400ms; the returned disposer clears the interval
    // (query() calls it on unmount/dispose — no leaked timer).
    stream: (): LiveSource<Tick> => ({
      subscribe(sig: LiveSignal<Tick>): () => void {
        let n = 0
        const id = setInterval(() => {
          n += 1
          sig.set({ kind: 'ok', value: { n, at: new Date().toISOString() } })
        }, 400)
        return () => clearInterval(id)
      },
    }),
  },
}
