// demo.server.ts — the SERVER resolution of the app's demo clients.
//
// The synthetic SSR build aliases `@app/clients/demo` to THIS module. Each
// client method returns a `LiveSource<T>` whose `subscribe` resolves the value
// over the host-dispatch wire (`call(op)` → `op_ship_call` → the host's
// Dispatch → deck's DataPort), instead of the browser's fetch/SSE. The typed
// shape matches `./demo.ts` (the client/type module) exactly — same contract,
// host-backed resolution.

import type { LiveSource, LiveSignal } from '@ship/sail-runtime'
import { call } from 'ship'
import type { ShipClient, ApiClient, TodoList, DemoConfig, Tick, BoomOk } from './demo'

/** A one-shot host-dispatch `LiveSource<T>`: ask the host for the op value,
 *  push `ok`/`error` into the live signal. The host owns the data. */
function hostSource<T>(op: string, args: Record<string, unknown> = {}): LiveSource<T> {
  return {
    subscribe(sig: LiveSignal<T>): () => void {
      let cancelled = false
      Promise.resolve(call(op, args))
        .then((value) => {
          if (cancelled) return
          // The host returns `{ error }` data on a classified failure.
          const v = value as { error?: string }
          if (v && typeof v === 'object' && typeof v.error === 'string') {
            sig.set({ kind: 'error', error: new Error(v.error) })
          } else {
            sig.set({ kind: 'ok', value: value as T })
          }
        })
        .catch((err: unknown) => {
          if (!cancelled) sig.set({ kind: 'error', error: err instanceof Error ? err : new Error(String(err)) })
        })
      return () => {
        cancelled = true
      }
    },
  }
}

export function makeShip(): ShipClient {
  return {
    demo: {
      todos: () => hostSource<TodoList>('ship.demo.todos'),
      config: () => hostSource<DemoConfig>('ship.demo.config'),
      boom: () => hostSource<BoomOk>('ship.demo.boom'),
    },
  }
}

export function makeApi(): ApiClient {
  return {
    demo: {
      clock: () => hostSource<Tick>('api.demo.clock'),
    },
  }
}
