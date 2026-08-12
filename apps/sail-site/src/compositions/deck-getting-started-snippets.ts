// Code snippets for the Deck getting-started page. Plain data module — never
// scope-scanned by the compiler.

import { signal } from '@ship/sail-runtime'
export const pageExample = `<!-- routes/page.sail -->
<main>
  <h1>Hello, deck.</h1>
</main>`

export const controllerExample = `<!-- routes/page.sail — one file: state + template.
     controller/signal are ambient inside <controller> — no import. -->
<controller>
  export default controller(() => {
    const count = signal(0)
    return { count }
  })
</controller>

<main>
  <h1>Hello, deck.</h1>
  <button on:click={() => count.set(count() + 1)}>
    clicked {count()} times
  </button>
</main>`

export const devCmd = `$ sail dev --project .
  ▸ serving http://127.0.0.1:7878
  ▸ watching routes/ for changes`
