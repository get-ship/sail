// Code snippets for the SSR + hydration page. Kept OUT of the controller so
// the compiler's textual scope scan isn't fooled by the `controller({ ... })`
// / `mount(...)` / `hydrate` patterns inside these strings. The pair the page
// itself forms (page.sui + controller.ts) is what the in-repo sui-compiler
// checks; these are illustrative.
export const code = {
  // scope() is isomorphic — it runs server-side to render and client-side to
  // hydrate. mount() is the client-only escape hatch for browser APIs.
  guard: `import { controller, signal } from '@ship/sui-runtime'

export default controller({
  // scope() runs on BOTH sides: on the server to render first-paint
  // HTML, and on the client to hydrate it. Keep it pure — reaching for
  // document or window here throws in the V8 render isolate.
  scope: () => ({ width: signal(0) }),

  // mount() runs ONLY on the client, after the DOM exists. Browser-only
  // APIs — window, document, observers, event listeners — belong here.
  mount(state, host) {
    state.width = window.innerWidth
  },
})`,

  // The serialized output the daemon ships. Each dynamic position carries the
  // marker the client hydrate() reads to re-attach reactivity in place.
  markers: `<!-- server-rendered HTML carries the hydration marker contract -->
<p><!--sui:text:b0-->42<!--/sui--> visitors online</p>

<button on:click="b1">Refresh</button>`,
}
