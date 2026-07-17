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
