// Code snippets for the Deck getting-started page. Plain data module — never
// scope-scanned by the compiler.

export const pageExample = `<!-- routes/page.sui -->
<main>
  <h1>Hello, deck.</h1>
</main>`

export const controllerExample = `<!-- routes/page.sui — one file: state + template -->
<controller>
  import { controller, signal } from '@ship/sui-runtime'

  export default controller({
    setup: () => {
      const count = signal(0)
      return { count }
    },
  })
</controller>

<main>
  <h1>Hello, deck.</h1>
  <button on:click={() => count.set(count() + 1)}>
    clicked {count()} times
  </button>
</main>`

export const devCmd = `$ deck dev --project .
  ▸ serving http://127.0.0.1:7878
  ▸ watching routes/ for changes`
