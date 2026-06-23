// The conformance app's demo data clients, as REAL typed ESM modules.
//
// In the ambient/`BUILTINS` path these are injected at runtime via
// `ctx.clients` (the `<client name="…">` seam). The synthetic path replaces
// that with a real import named by the app manifest: `ship` → `makeShip`,
// `api` → `makeApi`. tsgo/tsc then type-checks every `<data source="ship.…">`
// and template read against THESE types — there is no ambient passthrough.
//
// The SSR build aliases `./clients/demo` to a host-dispatch-backed server
// entry (see the deck view-host SSR path), so server-side these methods
// resolve through `op_ship_call`; in the browser they fetch the deck op
// surface. Either way the typed shape is the one source of truth.

import type { LiveSource, LiveSignal } from '@ship/sui-runtime'

/** The deck op-surface URL prefix (mirrors `@ship/sdk-client`'s `DECK_PREFIX`,
 *  inlined so the demo client has no codegen-bearing dependency). */
const DECK_PREFIX = '/_deck'

/** One todo row. */
export interface Todo {
  id: string
  text: string
  done: boolean
}

/** The `ship.demo.todos()` payload. */
export interface TodoList {
  items: Todo[]
}

/** The `api.demo.clock()` payload. */
export interface Tick {
  now: string
}

/** A demo app config blob (the non-live `<data>` shape exerciser). */
export interface DemoConfig {
  title: string
}

/** A one-shot `LiveSource<T>`: fetch the op value once, push `ok`/`error`.
 *  The mux/SSE live-update path is the production client's concern; this
 *  keeps the conformance client real but minimal. */
function once<T>(op: string, args: Record<string, unknown> = {}): LiveSource<T> {
  return {
    subscribe(sig: LiveSignal<T>): () => void {
      let cancelled = false
      const qs = new URLSearchParams({ args: JSON.stringify(args) }).toString()
      fetch(`${DECK_PREFIX}/op/${op}?${qs}`, { credentials: 'same-origin' })
        .then((r) => r.json() as Promise<T>)
        .then((value) => {
          if (!cancelled) sig.set({ kind: 'ok', value })
        })
        .catch((err: unknown) => {
          if (!cancelled) {
            sig.set({ kind: 'error', error: err instanceof Error ? err : new Error(String(err)) })
          }
        })
      return () => {
        cancelled = true
      }
    },
  }
}

/** The `ship` client: app-domain demo ops. */
export interface ShipClient {
  demo: {
    todos(): LiveSource<TodoList>
    config(): LiveSource<DemoConfig>
  }
}

/** The `api` client: a second, distinct demo domain. */
export interface ApiClient {
  demo: {
    clock(): LiveSource<Tick>
  }
}

/** Construct the `ship` client (manifest `construct: true`). */
export function makeShip(): ShipClient {
  return {
    demo: {
      todos: () => once<TodoList>('demo.todos'),
      config: () => once<DemoConfig>('demo.config'),
    },
  }
}

/** Construct the `api` client (manifest `construct: true`). */
export function makeApi(): ApiClient {
  return {
    demo: {
      clock: () => once<Tick>('demo.clock'),
    },
  }
}
