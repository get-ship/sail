// The shared store behind the State chapter's live demo: two
// <store-counter> and one <store-total> all resolve this ONE
// definition via `useStore(tally)`. Because element scopes form a
// tree rooted at the page's client root, all three get the SAME
// instance — clicking either counter moves every reader.
import { defineStore, signal } from '@ship/sail-runtime'

export const tally = defineStore('docs.tally', () => {
  const count = signal(0)
  return {
    count,
    add: (n: number) => count.set((c) => c + n),
    reset: () => count.set(0),
  }
})
