// #165 slice 6 — confinement overhead bench, in a real browser. Because the
// shell memoizes the confined module per view, confinement cost is ONE-TIME:
// lockdown() at boot + confineViewModule() once per view (then cached → per-mount
// is parity with the unconfined dynamic-import cache). This measures both
// one-time costs on a real compiled view. Result on `window.__bench`.

import { buildViewEndowments, confineViewModule, lockdownRealm } from '../../host/src/index.ts'
import * as runtime from '../../../sail/runtime/src/index.ts'
import * as dom from '../../../sail/dom/src/index.ts'

declare const __GIT_VIEW_SOURCE__: string

interface Bench {
  viewBytes: number
  lockdownMs: number
  confineAvgMs: number
  confineMinMs: number
  confineMaxMs: number
  iterations: number
}

function run(): Bench {
  const t0 = performance.now()
  lockdownRealm()
  const lockdownMs = performance.now() - t0

  const endow = buildViewEndowments({
    runtime: runtime as unknown as Record<string, unknown>,
    dom: dom as unknown as Record<string, unknown>,
    ship: {},
    doc: document,
  })

  // Warm once (first eval pays JIT/parse warmup), then measure.
  confineViewModule(__GIT_VIEW_SOURCE__, endow)
  const N = 25
  const times: number[] = []
  for (let i = 0; i < N; i++) {
    const s = performance.now()
    confineViewModule(__GIT_VIEW_SOURCE__, endow)
    times.push(performance.now() - s)
  }
  const sorted = [...times].sort((a, b) => a - b)
  const avg = times.reduce((a, b) => a + b, 0) / N

  return {
    viewBytes: __GIT_VIEW_SOURCE__.length,
    lockdownMs: Math.round(lockdownMs * 100) / 100,
    confineAvgMs: Math.round(avg * 100) / 100,
    confineMinMs: Math.round(sorted[0]! * 100) / 100,
    confineMaxMs: Math.round(sorted[N - 1]! * 100) / 100,
    iterations: N,
  }
}

;(window as unknown as { __bench: Bench }).__bench = run()
