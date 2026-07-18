// The clients seam for the playground.
//
//  • Imports the deck theme CSS so it loads as a global, document-level
//    stylesheet (the app wears the deck palette, not the ship one).
//  • Exposes a tiny PERSISTED store — a `visits` counter that survives
//    client-side navigation AND reload. Route signals reset on nav (each route
//    remounts); this lives above the router, so it's how you demonstrate
//    durable state. Read it in any view as `ctx.clients.store`.
import './deck-theme.css'
import './base.css'
import { signal } from '@ship/sail-runtime'

const KEY = 'deck-playground:v1'

function read(): { visits: number } {
  try {
    return { visits: 0, ...(JSON.parse(localStorage.getItem(KEY) ?? '{}') as object) }
  } catch {
    return { visits: 0 }
  }
}

const visits = signal<number>(read().visits)

function persist(): void {
  try {
    localStorage.setItem(KEY, JSON.stringify({ visits: visits() }))
  } catch {
    /* private mode / no storage — fine, the signal still works in-session */
  }
}

// The provider declares its OWN key on the SAIL-local `Clients` port — the
// framework names no host shape. This is what makes `ctx.clients.store` (and
// the bare `store` the runtime spreads into scope) PRECISELY typed rather than
// an ambient any: since #109 the generated `Scope` types each declared client
// through this port, so an app that exposes a name at the clients seam and
// never augments here gets a type error at the read, not silence.
declare module '@ship/sail-runtime' {
  interface Clients {
    store: typeof store
  }
}

export const store = {
  /** Survives client-side navigation + reload (lives above the router). */
  visits,
  noteVisit(): void {
    visits.set(visits() + 1)
    persist()
  },
  reset(): void {
    visits.set(0)
    persist()
  },
}
