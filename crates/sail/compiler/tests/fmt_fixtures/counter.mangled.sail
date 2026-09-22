<controller>
import { controller, signal } from '@ship/sail-runtime'
import type { Scope } from './counter.sail'

export default controller<Scope>((ctx) => ({ count: signal(0) }))
</controller>

<div>
<p>count: <span data-test="value">{count()}</span></p>
  <button   data-test="inc"   on:click={() => count.set(count() + 1)}  >
    increment
  </button>
<button data-test="reset" on:click={() => count.set(0)}>reset</button>
</div>
