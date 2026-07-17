// ssr-entry.ts — the SERVER ENTRY: render a synthesized SUI view to an HTML
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

// Flush async `live()` sources before serializing (must-build #5):
// a `<data live source="api.x()">` resolves over the host-dispatch wire — an
// async op. Mounting renders the `loading` state synchronously; the wire's
// in-flight counter (`globalThis.__shipPending()`, maintained by the SDK's
// `call`) lets us drain DETERMINISTICALLY: keep yielding while host calls are
// outstanding, then a few quiet turns so each resolved op's `.then` → live
// signal `.set` → bound effect → server-DOM write completes. Bounded so a
// never-resolving source can't hang first paint.
const pending = (g.__shipPending as undefined | (() => number)) ?? (() => 0)
let quiet = 0
for (let i = 0; i < 1000 && quiet < 3; i++) {
  await Promise.resolve()
  if (pending() > 0) quiet = 0
  else quiet++
}
const html = dom.html(container)
dispose()

emit({ html, routeParams: route?.params ?? null })
