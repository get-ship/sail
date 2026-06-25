// Closure / effect-retention signal for the leak-budget harness — the third
// growth class, beyond the synchronous DOM signals in `leak-budget.ts`.
//
// WeakRef + FinalizationRegistry catch what the DOM-only signals cannot: a node
// that IS removed (bodyDelta=0) and mints no new element, yet survives teardown
// because a closure (an undisposed effect, an orphaned subscription, an
// un-removed listener) still captures it. After each mount/unmount cycle we drop
// the local reference, force GC (when `global.gc` is exposed), and count probes
// still `.deref()`-able.
//
// LIMITATION: requires `global.gc` — run vitest with
// `--node-options=--expose-gc` (the test-setup config wires `execArgv:
// ['--expose-gc']`). Without it the measure is INCONCLUSIVE, not silently green:
// it reports `gcUnavailable: true` and `assertNoRetention` will not claim a pass
// it could not prove.

import { measureMountLeak, type LeakBudgetOpts, type LeakReport } from './leak-budget'

export interface RetentionOpts extends LeakBudgetOpts {
  /** GC passes to attempt after teardown before counting survivors. Each pass
   *  yields to a macrotask so FinalizationRegistry callbacks can drain. Default 5. */
  gcPasses?: number
}

export interface RetentionFields {
  /** Probed objects still reachable (`.deref()` non-undefined) after teardown +
   *  forced GC. 0 means every probed node/closure was collected. */
  retainedAfterTeardown: number
  /** How many probes were registered (one per cycle that yielded a probe). */
  retainedProbed: number
  /** True when `global.gc` was not exposed — the retention signal is INCONCLUSIVE
   *  (not a pass). Run with `--expose-gc` to make it conclusive. */
  gcUnavailable: boolean
}

/** Stand-alone retention report from `measureRetention`. */
export interface RetentionReport extends RetentionFields {
  iterations: number
  /** Residual `document.body` children vs the pre-run baseline (a removed-but-
   *  retained node leaves 0 here while `retainedAfterTeardown` > 0). */
  bodyDelta: number
  ok: boolean
}

/** A `LeakReport` plus the closure/effect-retention signal (async-only — GC and
 *  FinalizationRegistry drain need macrotask yields). */
export interface LeakReportWithRetention extends LeakReport, RetentionFields {}

/** A retention cycle returns either a bare teardown (no probe — that cycle
 *  contributes nothing to the retention count) or a teardown paired with the
 *  `probe` object whose collectability after teardown is tracked. When the cycle
 *  mounts a DOM node, pass that node as `probe`: if a closure pins it, it will
 *  still `.deref()` after GC. */
export type RetentionCycle = (() => void) | { teardown: () => void; probe?: object }

/** True when `global.gc` is exposed (run vitest with `--expose-gc`). The
 *  retention signal is only conclusive when this is true. */
export function gcAvailable(): boolean {
  return typeof (globalThis as { gc?: () => void }).gc === 'function'
}

/** Run GC (if exposed) over `passes` rounds, yielding to a macrotask between
 *  rounds so FinalizationRegistry callbacks can drain. Before collecting, churns
 *  a little throwaway garbage to evict the most-recent probe from the nursery /
 *  CPU registers — without this, V8 keeps the LAST cycle's object alive even
 *  after its closures are gone, which would spuriously read as a 1-node "leak".
 *  No-op when gc is absent. */
export async function forceGc(passes = 5): Promise<void> {
  const gc = (globalThis as { gc?: () => void }).gc
  if (typeof gc !== 'function') return
  for (let i = 0; i < Math.max(1, passes); i++) {
    // Flush registers/nursery: a short-lived allocation the optimizer can't
    // fold away, so the prior frame's young object is no longer rooted.
    let sink: unknown = []
    for (let j = 0; j < 2000; j++) sink = { j, sink }
    void sink
    sink = undefined
    gc()
    // Yield so the engine can run FinalizationRegistry callbacks + settle.
    await new Promise<void>((r) => setTimeout(r, 0))
  }
}

function normalizeRetentionCycle(out: unknown): { teardown: () => void; probe?: object } {
  if (typeof out === 'function') return { teardown: out as () => void }
  const o = out as { teardown: () => void; probe?: object }
  return o.probe !== undefined ? { teardown: o.teardown, probe: o.probe } : { teardown: o.teardown }
}

/**
 * Measure closure/effect retention across `iterations` mount/unmount cycles.
 *
 * Each cycle's `probe` is wrapped in a WeakRef and registered with a
 * FinalizationRegistry; after every teardown the local reference is dropped, and
 * after the whole run we `forceGc()` and count probes still `.deref()`-able.
 * A clean component collects all probes (`retainedAfterTeardown === 0`); a
 * closure-retain pins them (`> 0`) even though `bodyDelta` stays 0.
 *
 * Requires `global.gc`. Without it the run still completes but sets
 * `gcUnavailable: true` and `ok` is left true (we will not fail a measure we
 * could not perform — the caller / `assertNoRetention` surfaces the limitation).
 */
export async function measureRetention(
  cycle: () => RetentionCycle,
  opts: RetentionOpts = {},
): Promise<RetentionReport> {
  const iterations = Math.max(2, opts.iterations ?? 20)
  const gcPasses = opts.gcPasses ?? 5
  const baseline = document.body.childElementCount

  const refs: WeakRef<object>[] = []
  const registry = new FinalizationRegistry<number>(() => {
    /* presence of the registry is what keeps probes weakly tracked; the
     * heldValue is unused — `.deref()` is the survivor count source of truth. */
  })

  // Run the cycles in a SEPARATE frame so every per-iteration local (the cycle's
  // `teardown` + `probe` references) is dead by the time we GC — otherwise the
  // final iteration's object stays rooted on this function's stack and reads as
  // a spurious 1-node leak. Only the WeakRefs (intentionally weak) survive here.
  function runCycles(): void {
    for (let i = 0; i < iterations; i++) {
      const { teardown, probe } = normalizeRetentionCycle(cycle())
      if (probe != null && typeof probe === 'object') {
        refs.push(new WeakRef(probe))
        registry.register(probe, i)
      }
      teardown()
    }
  }
  runCycles()

  const gcUnavailable = !gcAvailable()
  await forceGc(gcPasses)

  let retainedAfterTeardown = 0
  for (const ref of refs) {
    if (ref.deref() !== undefined) retainedAfterTeardown++
  }

  const bodyDelta = document.body.childElementCount - baseline
  // ok requires both DOM-clean AND nothing retained — but only assert the
  // retention half when we could actually GC.
  const ok = bodyDelta === 0 && (gcUnavailable || retainedAfterTeardown === 0)
  return {
    iterations,
    bodyDelta,
    retainedAfterTeardown,
    retainedProbed: refs.length,
    gcUnavailable,
    ok,
  }
}

/**
 * Assert no closure/effect retention — throws a descriptive error if probed
 * nodes survive teardown after GC. When `global.gc` is unavailable the measure
 * is inconclusive: it does NOT throw (can't fail what it couldn't run) but the
 * returned report carries `gcUnavailable: true` so the caller sees it.
 */
export async function assertNoRetention(
  cycle: () => RetentionCycle,
  opts: RetentionOpts = {},
): Promise<RetentionReport> {
  const r = await measureRetention(cycle, opts)
  if (!r.gcUnavailable && r.retainedAfterTeardown > 0) {
    throw new Error(
      `closure/effect retention over ${r.iterations} cycles — ` +
        `${r.retainedAfterTeardown}/${r.retainedProbed} probed nodes still ` +
        `reachable after teardown + GC (bodyDelta=${r.bodyDelta}; the node was ` +
        `removed from the DOM but a closure — an undisposed effect, an orphaned ` +
        `subscription, an un-removed listener — still holds it. This is the ` +
        `effect-retention class the #258 registry signal cannot see).`,
    )
  }
  return r
}

/**
 * Unified async report: the synchronous mount-leak signals (bodyDelta +
 * define-count) PLUS the retention signal in one pass. Use this when you want a
 * single verdict over all three classes. The DOM-node the cycle creates is the
 * default probe — return `{ teardown, probe }` to track a specific object.
 */
export async function measureMountLeakAsync(
  cycle: () => RetentionCycle,
  opts: RetentionOpts = {},
): Promise<LeakReportWithRetention> {
  // Run the synchronous signals first (they restore `customElements.define`),
  // then the retention pass — both over the same cycle, fresh each time.
  const sync = measureMountLeak(() => {
    const out = normalizeRetentionCycle(cycle())
    return out.teardown
  }, opts)
  const retention = await measureRetention(cycle, opts)

  const ok = sync.ok && retention.ok
  return {
    iterations: sync.iterations,
    newDefines: sync.newDefines,
    definesPerIteration: sync.definesPerIteration,
    bodyDelta: sync.bodyDelta,
    retainedAfterTeardown: retention.retainedAfterTeardown,
    retainedProbed: retention.retainedProbed,
    gcUnavailable: retention.gcUnavailable,
    ok,
  }
}
