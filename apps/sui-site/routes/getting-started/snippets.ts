// Code snippets for the getting-started page. Kept out of the controller so the
// compiler's scope scan isn't fooled by `return {…}` / `signal(…)` patterns. Every
// SUI sample here is compiled by the build
// (crates/sui/compiler/tests/site_snippets_compile.rs).
export const code = {
  install: `# npm, pnpm, yarn, or bun — SUI is just packages
npm install @ship/sui-runtime @ship/sui-primitives`,

  component: `<!-- greeting.sui — a component is a <component> block + markup -->
<component>{
  props: {
    who: 'world',
  },
}</component>

<!-- no <style> needed: text is styled from the type scale,
     and layout/surface come from sx + the layout keywords -->
<p>Hello, {who()}.</p>`,

  compile: `# lower the .sui to a standard custom-element module
sui-compiler build greeting.sui --component-name sui-greeting
# → greeting.js (imports @ship/sui-runtime, registers <sui-greeting>)`,

  use: `<!-- importing the compiled module registers the element -->
<script type="module">
  import './greeting.js'
</script>

<!-- then it's a first-class HTML element — use it anywhere -->
<sui-greeting who="Ada"></sui-greeting>`,

  interactive: `<!-- counter.sui — when there's state, add an inline <controller>
     block. One file: logic and markup live together (SFC). -->
<controller>
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => ({ count: signal(0) }),
})
</controller>

<button on:click={() => count.set((n) => n - 1)}>−</button>
<output>{count()}</output>
<button on:click={() => count.set((n) => n + 1)}>+</button>`,

  deck: `# deck is optional — an app framework that drives the compiler
# for a whole app: bundling, a dev loop, routing, server rendering.
deck dev --project .`,
}
