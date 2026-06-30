// Code snippets for the reactivity page. Kept OUT of the controller so the
// compiler's scope scan isn't fooled by `return {…}` / `signal(…)` patterns
// inside these strings.
export const code = {
  signal: `import { signal } from '@ship/sui-runtime'

const count = signal(0)

count()        // read  → 0
count.set(5)   // write → 5
count()        // read  → 5`,
  derived: `import { signal, derived } from '@ship/sui-runtime'

const count = signal(2)
const doubled = derived(() => count() * 2)

doubled()      // 4
count.set(10)
doubled()      // 20  — recomputed lazily, only when read`,
  effect: `import { signal, effect } from '@ship/sui-runtime'

const count = signal(0)

effect(() => {
  console.log('count is', count())
})              // logs "count is 0" immediately

count.set(1)    // logs "count is 1" — re-runs on change`,
  template: `<!-- counter.sui -->
<button on:click={() => count.set(count() - 1)}>−</button>
<output>{count()}</output>
<button on:click={() => count.set(count() + 1)}>+</button>`,
}
