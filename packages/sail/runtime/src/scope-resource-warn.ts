// scope-resource-warn — dev-mode leak detector built on effect-scope.
//
// Problem this solves: a mount opens an EventSource / MutationObserver
// / setInterval / addEventListener (etc.) inside `withScope`, but the
// allocator forgets to register a cleanup. The scope.dispose() runs,
// nothing closes the resource, and it leaks across navigations until
// the tab OOMs. We learned this the hard way with the `live()` leak
// (commit 87cf3223) and the discarded `mount()` cleanups (commit
// 0efea6d7).
//
// What this module does: opt-in instrumentation that wraps the
// browser's resource constructors WHILE a scope is active and tracks
// allocations per scope. On `scope.dispose()`, any resource the
// caller didn't release is logged at warn level.
//
// Production behavior: don't import this file. The runtime barrel
// only re-exports it under a `dev` namespace; building production
// bundles with the dev flag off keeps the wrappers out.
//
// Implementation notes:
// - Wraps the GLOBAL constructors (EventSource, MutationObserver,
//   ResizeObserver, IntersectionObserver) via assignment to
//   `globalThis.X = wrapped`. Ref-counted: each `enableLeakWarn`
//   increments; restores the originals when the count hits zero.
// - For setInterval/setTimeout we patch `globalThis.setInterval`/
//   `clearInterval`. We DON'T patch addEventListener — too noisy and
//   most listener leaks come through the wrapped constructors anyway.
// - Each allocation looks up the current scope (via `currentScope`)
//   at construction time and stamps the resource. On scope dispose
//   we walk the per-scope set; anything still alive is logged.

import { currentScope, onCleanup, type EffectScope } from './effect-scope'

interface ResourceRecord {
  kind: string
  closed: boolean
  stack?: string | undefined
}

const RECORDS_BY_SCOPE = new WeakMap<EffectScope, Set<ResourceRecord>>()

function recordsFor(scope: EffectScope): Set<ResourceRecord> {
  let s = RECORDS_BY_SCOPE.get(scope)
  if (!s) {
    s = new Set()
    RECORDS_BY_SCOPE.set(scope, s)
    // Walk the records on dispose and log any still-open resources.
    scope.cleanup(() => {
      const open = [...s!].filter(r => !r.closed)
      if (open.length === 0) return
      const grouped = open.reduce<Record<string, number>>((acc, r) => {
        acc[r.kind] = (acc[r.kind] || 0) + 1
        return acc
      }, {})
      console.warn(
        '[sail/scope] mount disposed with open resources:',
        Object.entries(grouped)
          .map(([kind, n]) => `${n}× ${kind}`)
          .join(', '),
        '\n  (one stack trace per kind below)',
      )
      const sampleByKind = new Map<string, string | undefined>()
      for (const r of open) {
        if (!sampleByKind.has(r.kind)) sampleByKind.set(r.kind, r.stack)
      }
      for (const [kind, stack] of sampleByKind) {
        console.warn(`[sail/scope] sample ${kind} allocation site:\n${stack ?? '(no stack)'}`)
      }
    })
  }
  return s
}

function track(kind: string): ResourceRecord | null {
  const scope = currentScope()
  if (!scope) return null
  const rec: ResourceRecord = {
    kind,
    closed: false,
    stack: new Error().stack?.split('\n').slice(2, 6).join('\n'),
  }
  recordsFor(scope).add(rec)
  return rec
}

let installCount = 0
let restore: (() => void) | null = null

/** Enable dev-mode leak warnings. Idempotent — multiple callers
 *  share one installation, and the wrappers come down only when
 *  every caller has called the returned restore fn. Returns a
 *  disposer; recommend wiring it via `onCleanup` so test scopes
 *  uninstall cleanly. */
export function enableLeakWarn(): () => void {
  installCount++
  if (installCount === 1) restore = installWrappers()
  let disposed = false
  return () => {
    if (disposed) return
    disposed = true
    installCount--
    if (installCount === 0 && restore) {
      restore()
      restore = null
    }
  }
}

function installWrappers(): () => void {
  const restorers: Array<() => void> = []

  // EventSource
  if (typeof globalThis.EventSource === 'function') {
    const Orig = globalThis.EventSource
    class WrappedES extends Orig {
      private __rec: ResourceRecord | null
      constructor(url: string | URL, init?: EventSourceInit) {
        super(url, init)
        this.__rec = track('EventSource')
      }
      override close(): void {
        if (this.__rec) this.__rec.closed = true
        super.close()
      }
    }
    globalThis.EventSource = WrappedES as typeof EventSource
    restorers.push(() => { globalThis.EventSource = Orig })
  }

  // MutationObserver
  if (typeof globalThis.MutationObserver === 'function') {
    const Orig = globalThis.MutationObserver
    class Wrapped extends Orig {
      private __rec: ResourceRecord | null
      constructor(cb: MutationCallback) {
        super(cb)
        this.__rec = track('MutationObserver')
      }
      override disconnect(): void {
        if (this.__rec) this.__rec.closed = true
        super.disconnect()
      }
    }
    globalThis.MutationObserver = Wrapped as typeof MutationObserver
    restorers.push(() => { globalThis.MutationObserver = Orig })
  }

  // ResizeObserver
  if (typeof globalThis.ResizeObserver === 'function') {
    const Orig = globalThis.ResizeObserver
    class Wrapped extends Orig {
      private __rec: ResourceRecord | null
      constructor(cb: ResizeObserverCallback) {
        super(cb)
        this.__rec = track('ResizeObserver')
      }
      override disconnect(): void {
        if (this.__rec) this.__rec.closed = true
        super.disconnect()
      }
    }
    globalThis.ResizeObserver = Wrapped as typeof ResizeObserver
    restorers.push(() => { globalThis.ResizeObserver = Orig })
  }

  // IntersectionObserver
  if (typeof globalThis.IntersectionObserver === 'function') {
    const Orig = globalThis.IntersectionObserver
    class Wrapped extends Orig {
      private __rec: ResourceRecord | null
      constructor(cb: IntersectionObserverCallback, init?: IntersectionObserverInit) {
        super(cb, init)
        this.__rec = track('IntersectionObserver')
      }
      override disconnect(): void {
        if (this.__rec) this.__rec.closed = true
        super.disconnect()
      }
    }
    globalThis.IntersectionObserver = Wrapped as typeof IntersectionObserver
    restorers.push(() => { globalThis.IntersectionObserver = Orig })
  }

  // setInterval / clearInterval — record the timer id, mark closed on clear.
  const intervalRecords = new Map<unknown, ResourceRecord>()
  const origSetInterval = globalThis.setInterval
  const origClearInterval = globalThis.clearInterval
  ;(globalThis as { setInterval: typeof setInterval }).setInterval = ((
    handler: TimerHandler,
    timeout?: number,
    ...args: unknown[]
  ) => {
    const id = origSetInterval(handler as () => void, timeout, ...args)
    const rec = track('setInterval')
    if (rec) intervalRecords.set(id, rec)
    return id
  }) as typeof setInterval
  ;(globalThis as { clearInterval: typeof clearInterval }).clearInterval = ((
    id?: number,
  ) => {
    if (id !== undefined) {
      const rec = intervalRecords.get(id)
      if (rec) {
        rec.closed = true
        intervalRecords.delete(id)
      }
    }
    return origClearInterval(id)
  }) as typeof clearInterval
  restorers.push(() => {
    globalThis.setInterval = origSetInterval
    globalThis.clearInterval = origClearInterval
  })

  return () => {
    for (const r of restorers) r()
  }
}

/** Bridge for callers that want to enable leak warnings inside a
 *  specific scope (e.g. tests). Calls `enableLeakWarn`; registers
 *  the disposer via `onCleanup` so the wrappers come down when the
 *  scope ends. */
export function enableLeakWarnInScope(): void {
  const dispose = enableLeakWarn()
  onCleanup(dispose)
}
