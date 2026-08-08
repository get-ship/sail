<controller>
// Controller signal the HMR spec drives off default, then edits this file on
// disk and asserts the value survives the soft reload.
export default controller(() => ({ count: signal(0) }))
</controller>

<div>
  <h1>HMR original</h1>
  <p>Count: <strong>{count()}</strong></p>
  <button on:click={() => count.set(count() + 1)}>increment</button>
</div>
