// Code snippets for the reactivity page. Kept OUT of the controller so the
// compiler's scope scan isn't fooled by `return {…}` / `signal(…)` patterns
// inside these strings. Each snippet matches the real @ship/sail-runtime API
// (signal/derived/effect/batch/untrack); the .sail template compiles paired
// with a controller declaring `count`.
export const code = {
  graph: `<controller>
import { controller, signal, derived, effect } from '@ship/sail-runtime'
export default controller(() => {
  const count = signal(0)
  const doubled = derived(() => count() * 2)
  effect(() => console.log('count is', count()))
  return { count, doubled }
})
</controller>

<button on:click={() => count.set(count() + 1)}>+1</button>
<p>{count()} · doubled: {doubled()}</p>`,

  signal: `import { signal } from '@ship/sail-runtime'

const count = signal(0)

count()                 // read  → 0
count.set(5)            // write → 5
count.set((n) => n + 1) // functional update → 6`,

  derived: `import { signal, derived } from '@ship/sail-runtime'

const count = signal(2)
const doubled = derived(() => count() * 2)

doubled()      // 4
count.set(10)
doubled()      // 20 — recomputed lazily, only when read`,

  effect: `import { signal, effect } from '@ship/sail-runtime'

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

  escape: `import { signal, effect, batch, untrack } from '@ship/sail-runtime'

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

  template: `<!-- counter.sail -->
<button on:click={() => count.set((n) => n - 1)}>−</button>
<output>{count()}</output>
<button on:click={() => count.set((n) => n + 1)}>+</button>`,

  lifecycle: `import { controller, signal, mount } from '@ship/sail-runtime'

export default controller(() => {
  const size = signal({ w: 0, h: 0 })

  // mount runs once, after the element is in the DOM. el IS the element.
  mount((el) => {
    const ro = new ResizeObserver(([e]) => {
      size.set({ w: e.contentRect.width, h: e.contentRect.height })
    })
    ro.observe(el)
    return () => ro.disconnect()
  })

  return { size }
})`,

  composition: `// A COMPOSITION is a plain function over the primitives. No type, no
// registration, no base class. If it touches the DOM it is an ACTION; if it
// returns state it is a composition — that is the whole rule.
import { selection } from '@ship/sail-runtime'

export function useSelection(rows) {
  const { selected, isSelected, toggle } = selection(rows)
  return { selected, isSelected, toggle }
}`,
}
