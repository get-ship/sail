<controller>
import { controller, signal, derived } from '@ship/sail-runtime'

export default controller((ctx) => {
  const count = signal(0)
  const doubled = derived(() => count() * 2)
  const label = derived(() => `count is ${count()}`)
  const inc = () => count.set(count() + 1)
  return { count, doubled, label, inc }
})
</controller>

<div>
  <p>{label()}</p>
  <p>doubled: {doubled()}</p>
  <button on:click={inc}>+</button>
</div>
