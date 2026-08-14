// Owned defineStore fixture. Shape is a real store (one signal, one writer).
// Lives under tests/fixtures/stores — not compositions, not a live app.

import { defineStore, signal } from '@ship/sail-runtime'
export const tally = defineStore('lab.tally', () => {
  const clicks = signal(0)
  const add = () => clicks.set((n) => n + 1)
  return { clicks, add }
})
