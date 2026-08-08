// Constructed demo clients for route controllers — import roots for query() identity.
// `@app/clients/demo` SSR-aliases to demo.server.ts (host-dispatch).

import { makeApi, makeShip } from '@app/clients/demo'

export const api = makeApi()
export const ship = makeShip()

export type { ApiClient, ShipClient, Tick, TodoList, BoomOk, Todo, DemoConfig } from '@app/clients/demo'
