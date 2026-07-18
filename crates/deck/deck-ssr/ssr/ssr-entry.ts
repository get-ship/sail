// ssr-entry.ts — the SERVER ENTRY: render a synthesized SAIL view to an HTML
// string INSIDE ship-js (a deno_core/V8 isolate), with route + data resolved
// over the host-dispatch wire — never the browser.
//
// The server DOM lives in `@ship/sail-runtime/server` (`installServerDom`) — a
// shared, tested module, not an inline shim. Installing it as
// `globalThis.document` lets the SAME synthesized render that runs in the
// browser (`createElement`, `appendChild`, `bindText`, `bindFor`, …) run here
// unchanged and serialize its built tree to HTML — one isomorphic render
// artifact, not a second string-renderer.
//
// `import "ship"` is the one door: `call(op, args)` is the host-dispatch wire
// (`op_ship_call` → the host's `Dispatch`), `emit(v)` surfaces the result.

import { installServerDom, whenCacheSettled } from '@ship/sail-runtime'
import { mount } from './page.render.ts'
import { call, emit } from 'ship'

// Install the server DOM (globalThis.document / Node) before the render runs.
const dom = installServerDom()

// ── host-dispatch: resolve route server-side, park it for the route shim ─────
const route = (await call('route.current', {})) as { params?: Record<string, string> }
const g = globalThis as Record<string, unknown>
g.__shipServerRoute = route

// ── render the synthesized view to HTML ─────────────────────────────────────
const container = dom.container()
const dispose = mount(container)

// Flush async data sources before serializing (must-build #5): a controller
// `query(() => client.x())` resolves over the host-dispatch wire — an async
// op. Mounting renders the `loading` state synchronously.
// `whenCacheSettled()` (native-cache.ts) is the DETERMINISTIC completion
// signal: the cache brackets every PULL fetch from start to the moment its
// outcome is written into the entry's signal (cache entry `.set` → forwarding
// effect → outer signal `.set` → bound template effect → server-DOM write is
// synchronous inside that write, per the signal's own `flush()`), so once the
// promise resolves, first paint is guaranteed to reflect it — no macrotask-
// count-and-hope, no race with which queue a host op's completion lands on.
// Bounded so a source that never resolves (host never answers, or a stuck
// retry backoff) can't hang first paint forever: race against a deadline and
// serialize on `loading` if it fires first. 2s is generous for SSR-scoped
// host-dispatch (an in-process handler, not a real network hop).
const SETTLE_DEADLINE_MS = 2000
const g2 = globalThis as { setTimeout?: (fn: () => void, ms: number) => unknown }
const deadline = new Promise<void>((resolve) => {
  if (typeof g2.setTimeout === 'function') g2.setTimeout(resolve, SETTLE_DEADLINE_MS)
  else resolve() // no timers in this isolate: nothing to bound against, don't hang
})
await Promise.race([whenCacheSettled(), deadline])
const html = dom.html(container)
dispose()

emit({ html, routeParams: route?.params ?? null })
