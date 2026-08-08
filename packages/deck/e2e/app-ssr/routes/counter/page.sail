<controller>
// Signal initial value lives in JS so SSR cannot fold it: server paints structure
// + hydration anchor; client fills 0 on hydrate. Clicks prove reactivity survived.
export default controller(() => ({ count: signal(0) }))
</controller>

<div>
  <h1>Counter</h1>
  <p>Count: <strong>{count()}</strong></p>
  <button on:click={() => count.set(count() + 1)}>increment</button>
</div>
