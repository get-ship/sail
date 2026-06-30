// Landing page — drives the three live compositions. Every demo on the home
// proves a different SUI primitive in the page's own scope:
//   - the counter = signals + derived
//   - the todo card = two-way binding + array iteration + children slots
//   - the clock = effect with setInterval cleanup
//
// All under one controller so the cost of setting this up is the cost of
// shipping the page. No wasm compile needed.

import { controller, signal, derived, effect, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

const COUNTER_SRC = `<!-- counter.sui -->
<button on:click={() => dec()}>−</button>
<output>{count()}</output>
<button on:click={() => inc()}>+</button>
<p>doubled → {doubled()}</p>

// counter.controller.ts
const count = signal(0)
const doubled = derived(() => count() * 2)`

const TODO_SRC = `<sui-card>
  <h3 slot="title">Things to ship today</h3>
  <sui-input bind:value={draft}></sui-input>
  <sui-button on:click={add}>Add</sui-button>
  {#for it of items() key={it}
    <sui-checkbox checked={it === done()}
                 on:change={() => toggle(it)}>
      {it}
    </sui-checkbox>
  {/for}
</sui-card>`

const CLOCK_SRC = `// clock.controller.ts
const time = signal(new Date().toLocaleTimeString())
effect(() => {
  const id = setInterval(() => {
    time.set(new Date().toLocaleTimeString())
  }, 1000)
  return () => clearInterval(id)
})`

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const count = signal(0)
    const doubled = derived(() => count() * 2)
    const inc = (): void => count.set(count() + 1)
    const dec = (): void => count.set(Math.max(0, count() - 1))

    const draft = signal('')
    const items = signal<string[]>([])
    const done = signal<string | null>(null)
    const add = (): void => {
      const v = draft().trim()
      if (!v) return
      items.set([...items(), v])
      draft.set('')
    }
    const remove = (it: string): void => {
      items.set(items().filter((x) => x !== it))
      if (done() === it) done.set(null)
    }
    const toggle = (it: string): void => {
      done.set(done() === it ? null : it)
    }

    const time = signal(new Date().toLocaleTimeString())
    effect(() => {
      const id = setInterval(() => {
        time.set(new Date().toLocaleTimeString())
      }, 1000)
      return () => clearInterval(id)
    })

    return {
      count, doubled, inc, dec,
      draft, items, done, add, remove, toggle,
      time,
      counterSrc: COUNTER_SRC,
      todoSrc: TODO_SRC,
      clockSrc: CLOCK_SRC,
    }
  },
})
