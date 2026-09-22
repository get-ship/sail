// Leak-budget harness — detect UNBOUNDED GROWTH across repeated mount/unmount
// cycles, in-process (jsdom/happy-dom), with no browser and no OOM risk.
//
// Motivation: a long-lived webview heap balloons when a component's lifecycle
// accretes state it never releases. Three such classes are observable in-process
// and are what this helper guards:
//
//   1. NODE RETENTION — a teardown (unmount / disconnectedCallback) that leaves
//      detached roots in the document. Observable as `document.body` not
//      returning to its baseline child count each cycle.
//   2. REGISTRY GROWTH — the #258 class: a render path that mints a NEW custom
//      element per distinct source and `customElements.define`s it (the registry
//      is process-global and append-only — there is no undefine). Observable by
//      counting `define` calls across cycles: a define-once component settles to
//      zero new defines after warmup; a per-cycle minter grows ~1 define/cycle.
//   3. CLOSURE/EFFECT RETENTION — the subtle class the first two CANNOT see: the
//      node IS removed from the DOM (bodyDelta=0) and no new element is defined,
//      but an undisposed effect / orphaned subscription / un-removed listener
//      still captures it, so it never collects. It needs WeakRef +
//      FinalizationRegistry + a forced GC and is therefore async — it lives in
//      the sibling module `./leak-retention` (`measureRetention`,
//      `assertNoRetention`, `measureMountLeakAsync`), which reuses the
//      synchronous `measureMountLeak` below. This file owns signals (1) and (2),
//      which are synchronous and need no GC.

export interface LeakBudgetOpts {
  /** Mount/unmount cycles to run. Default 100. */
  iterations?: number
}

export interface LeakReport {
  iterations: number
  /** `customElements.define` calls observed AFTER the first (warmup) cycle. */
  newDefines: number
  /** newDefines / counted-cycles. >= 1 means the registry grows every cycle. */
  definesPerIteration: number
  /** Residual `document.body` children vs the pre-run baseline. */
  bodyDelta: number
  ok: boolean
}

/**
 * Run `cycle` `iterations` times — each call mounts once and returns its
 * teardown, which the helper invokes before the next cycle — and report growth.
 * The first cycle is treated as warmup so a legitimate define-once registration
 * isn't counted as a leak.
 */
export function measureMountLeak(
  cycle: () => () => void,
  opts: LeakBudgetOpts = {},
): LeakReport {
  const iterations = Math.max(2, opts.iterations ?? 100)
  const baseline = document.body.childElementCount

  const origDefine = customElements.define
  let newDefines = 0
  let counting = false
  ;(customElements as unknown as { define: typeof origDefine }).define = function (
    name: string,
    ctor: CustomElementConstructor,
    o?: ElementDefinitionOptions,
  ) {
    if (counting) newDefines++
    return origDefine.call(customElements, name, ctor, o)
  }

  try {
    for (let i = 0; i < iterations; i++) {
      const teardown = cycle()
      teardown()
      // Everything defined during the first cycle is the legitimate one-time
      // registration; count from the second cycle onward.
      if (i === 0) counting = true
    }
  } finally {
    ;(customElements as unknown as { define: typeof origDefine }).define = origDefine
  }

  const bodyDelta = document.body.childElementCount - baseline
  const counted = iterations - 1
  const definesPerIteration = newDefines / counted
  const ok = bodyDelta === 0 && definesPerIteration < 1
  return { iterations, newDefines, definesPerIteration, bodyDelta, ok }
}

/**
 * Assert no mount leak — throws a descriptive error if growth is detected, so it
 * reads naturally inside any test framework's `it(...)`. Returns the report.
 */
export function assertNoMountLeak(
  cycle: () => () => void,
  opts: LeakBudgetOpts = {},
): LeakReport {
  const r = measureMountLeak(cycle, opts)
  if (!r.ok) {
    throw new Error(
      `mount leak over ${r.iterations} cycles — ` +
        `bodyDelta=${r.bodyDelta} (detached roots left in document), ` +
        `newDefines=${r.newDefines} (${r.definesPerIteration.toFixed(2)}/cycle; ` +
        `>=1 means the custom-element registry grows every cycle — see #258).`,
    )
  }
  return r
}

// The third growth class — CLOSURE/EFFECT RETENTION (a node removed but pinned
// by an undisposed effect/subscription) — needs WeakRef + FinalizationRegistry
// + a forced GC and is therefore async. It lives in `./leak-retention`
// (`measureRetention`, `assertNoRetention`, `measureMountLeakAsync`) and reuses
// `measureMountLeak` above for the synchronous signals.
