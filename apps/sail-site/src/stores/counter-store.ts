// The State chapter's shared store. Deliberately tiny: one signal, one
// writer. Imported by <store-counter> AND the chapter page — that's the
// demonstration: neither knows about the other, both see the same state.

import { defineStore, signal } from '@ship/sail-runtime'
export const tally = defineStore('docs.tally', () => {
  const clicks = signal(0)
  const add = () => clicks.set((n) => n + 1)
  return { clicks, add }
})
