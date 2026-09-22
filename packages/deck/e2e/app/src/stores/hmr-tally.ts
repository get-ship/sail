// Shared store for the HMR edge spec: a page signal must survive an `@/` store
// edit remount. `storeBanner` is a plain string so restoreScope does not keep
// the old label after the module swaps.

import { defineStore, signal } from '@ship/sail-runtime'

export const storeBanner = 'store original'

export const hmrTally = defineStore('e2e.hmrTally', () => {
  const clicks = signal(0)
  const add = () => clicks.set(clicks() + 1)
  return { clicks, add }
})
