// Code snippets for the Getting-started page, kept OUT of the paired
// controller.ts on purpose: the SAIL compiler scans a route's controller for
// `signal(...)` to infer scope, and example code containing `signal(0)` would
// leak into it. A plain data module (not a controller) is never scope-scanned.

export const pageExample = `<!-- routes/page.sail -->
<main>
  <h1>Hello, deck.</h1>
  <button on:click={() => count.set(count() + 1)}>
    clicked {count()} times
  </button>
</main>`

export const controllerExample = `// routes/controller.ts
import { controller, signal } from '@ship/sail-runtime'

export default controller(() => {
  const count = signal(0)
  return { count }
})`

export const devCmd = `$ deck dev --project .
  ▸ serving http://127.0.0.1:7878
  ▸ watching routes/ for changes`
