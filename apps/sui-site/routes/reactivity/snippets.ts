// Code snippets for the reactivity page. Kept OUT of the controller so the
// compiler's scope scan isn't fooled by `return {…}` / `signal(…)` patterns
// inside these strings. Each snippet matches the real @ship/sui-runtime API
// (signal/derived/effect/batch/untrack); the .sui template compiles paired
// with a controller declaring `count`.
export const code = {
  signal: `import { signal } from '@ship/sui-runtime'

const count = signal(0)

count()                 // read  → 0
count.set(5)            // write → 5
count.set((n) => n + 1) // functional update → 6`,

  derived: `import { signal, derived } from '@ship/sui-runtime'

const count = signal(2)
const doubled = derived(() => count() * 2)

doubled()      // 4
count.set(10)
doubled()      // 20 — recomputed lazily, only when read`,

  effect: `import { signal, effect } from '@ship/sui-runtime'

const count = signal(0)

// Runs now, re-runs whenever a signal it read changes.
// Return a cleanup — it runs before the next run and on dispose,
// so the effect owns its resources (timers, sockets, listeners).
const dispose = effect(() => {
  const label = 'count=' + count()
  const id = setTimeout(() => console.log(label), 200)
  return () => clearTimeout(id)
})

count.set(1)   // clears the pending log, schedules a fresh one
// dispose()   // stop the effect entirely`,

  escape: `import { signal, effect, batch, untrack } from '@ship/sui-runtime'

const first = signal('Ada')
const last = signal('Lovelace')

// batch: coalesce writes into ONE update pass
batch(() => {
  first.set('Grace')
  last.set('Hopper')
})   // dependent effects run once, not twice

// untrack: read a signal without subscribing to it
effect(() => {
  console.log(first(), untrack(() => last()))
})   // re-runs when first changes — never when last does`,

  template: `<!-- counter.sui -->
<button on:click={() => count.set((n) => n - 1)}>−</button>
<output>{count()}</output>
<button on:click={() => count.set((n) => n + 1)}>+</button>`,
}
