// Snippets for /docs/concepts/state. Kept OUT of the controller so the
// compiler's scope scan isn't fooled by `return {…}` / `signal(…)` patterns
// inside these strings. The graph snippet IS the page's own controller — the
// source beside the live demo matches what the demo runs.
export const code = {
  graph: `<controller>
export default controller(() => {
  const count = signal(0)
  const doubled = derived(() => count() * 2)
  const log = signal<string[]>([])
  effect(() => {
    const n = count()
    log.set((prev) => [...prev.slice(-2), \`effect ran — count is now \${n}\`])
  })
  return { count, doubled, log }
})
</controller>`,

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

  effect: `import { signal, effect, debounce } from '@ship/sail-runtime'

const count = signal(0)
const status = signal('')

// Runs now; re-runs whenever a signal it reads changes.
// Debounced work is lifecycle-bound — cleared on re-run / dispose.
const dispose = effect(() => {
  const n = count()
  const push = debounce(() => { status.set(\`count: \${n}\`) }, 200)
  push()
})

count.set(1)   // cancels the pending update, schedules a fresh one
// dispose()   // stop the effect entirely`,

  batch: `import { batch } from '@ship/sail-runtime'

batch(() => {
  first.set('Grace')
  last.set('Hopper')
})
// fullName recomputes once; effects that read it run once`,

  untrack: `import { effect, untrack } from '@ship/sail-runtime'

effect(() => {
  const text = draft()                    // tracked — re-runs on edits
  const who = untrack(() => user().name)  // sampled — no subscription
  save(text, who)
})`,

  template: `<button on:click={() => count.set(count() + 1)}>+1</button>
<strong>{count()}</strong>
<strong>{doubled()}</strong>
<for {log() as line} key={line}>
  <p>{line}</p>
</for>`,

  // Inside <controller>, controller/signal/mount are ambient — no import.
  lifecycle: `export default controller(() => {
  const size = signal({ w: 0, h: 0 })

  // mount runs once, after the element is on the page. el IS the element.
  mount((el) => {
    const ro = new ResizeObserver(([entry]) => {
      size.set({ w: entry.contentRect.width, h: entry.contentRect.height })
    })
    ro.observe(el)
    return () => ro.disconnect()   // the undo, when the component leaves
  })

  return { size }
})`,

  // Plain modules are NOT ambient — import signal/defineStore as usual.
  storeDef: `// counter-store.ts — outside any component
import { defineStore, signal } from '@ship/sail-runtime'

export const tally = defineStore('docs.tally', () => {
  const clicks = signal(0)
  const add = () => clicks.set((n) => n + 1)
  return { clicks, add }
})`,

  // useStore is not ambient; controller/signal are.
  storeUse: `import { useStore } from '@ship/sail-runtime'
import { tally } from '@/stores/counter-store'

export default controller(() => {
  const { clicks, add } = useStore(tally)
  return { clicks, add }
})`,

  composition: `import { signal, interval } from '@ship/sail-runtime'

// A composition: state + the interval that maintains it, as one function.
export function ticker(ms: number) {
  const now = signal(Date.now())
  interval(() => now.set(Date.now()), ms)
  return now
}

// in any controller:
const now = ticker(1000)`,
}
