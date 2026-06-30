// Snippet shown beside the live counter on the overview. Kept out of the
// controller so the scope scan isn't fooled by the `signal(…)` / `derived(…)`
// patterns inside it.
export const COUNTER_SRC = `<!-- counter.sui -->
<button on:click={() => dec()}>−</button>
<output>{count()}</output>
<button on:click={() => inc()}>+</button>
<p>doubled → {doubled()}</p>

// counter.controller.ts
const count = signal(0)
const doubled = derived(() => count() * 2)`
