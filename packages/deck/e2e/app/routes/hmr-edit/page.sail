<controller>
import { controller, signal } from '@ship/sail-runtime'

// A controller signal — the HMR spec drives it to a non-default value, edits
// page.sail on disk, and asserts the value survives the soft reload.
export default controller(() => ({ count: signal(0) }))
</controller>

<div data-hmr-edit="true">
  <h1 data-hmr-heading="true">HMR original</h1>
  <p>Count: <strong>{count()}</strong></p>
  <button on:click={() => count.set(count() + 1)}>increment</button>
</div>
