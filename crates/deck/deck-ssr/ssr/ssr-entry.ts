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

import { installServerDom } from '@ship/sail-runtime'
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
// op. Mounting renders the `loading` state synchronously; the wire's
// in-flight counter (`globalThis.__shipPending()`, maintained by the SDK's
// `call`) lets us drain DETERMINISTICALLY: keep yielding while host calls are
// outstanding, then some quiet turns so each resolved op's `.then` → cache
// entry `.set` → forwarding effect → outer signal `.set` → bound template
// effect → server-DOM write completes. `query()` has a slightly deeper settle
// chain than the old `live()` (the cache entry + re-keying forwarding effect
// add hops), so the quiet budget is a touch more generous. Bounded so a
// never-resolving source can't hang first paint.
const pending = (g.__shipPending as undefined | (() => number)) ?? (() => 0)
// Warm-up: a few unconditional turns let a controller's eager fetch dispatch
// its host call before we start judging "quiet", so a source that hasn't
// bumped `pending()` yet isn't mistaken for "nothing outstanding".
for (let i = 0; i < 8; i++) await Promise.resolve()
let quiet = 0
for (let i = 0; i < 3000 && quiet < 12; i++) {
  await Promise.resolve()
  if (pending() > 0) quiet = 0
  else quiet++
}
const html = dom.html(container)
dispose()

emit({ html, routeParams: route?.params ?? null })
