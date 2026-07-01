// Code snippets for the getting-started page. Kept OUT of the controller so the
// compiler's scope scan isn't fooled by `return {…}` / `signal(…)` patterns.
// The .sui samples are compile-checked against the in-repo sui-compiler: the
// component compiles on its own; the interactive template compiles paired with
// its controller. Re-run the compiler before editing a sample.
export const code = {
  install: `# npm, pnpm, yarn, or bun — SUI is just packages
npm install @ship/sui-runtime @ship/sui-primitives`,

  component: `<!-- greeting.sui — a component is a <prop> + markup -->
<prop name="who" type="string" default="world" />

<style>
  :host { display: block; }
  p { color: var(--foreground); }
</style>

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

  interactive: `<!-- counter.sui — when there's state, add a controller -->
<button on:click={() => count.set((n) => n - 1)}>−</button>
<output>{count()}</output>
<button on:click={() => count.set((n) => n + 1)}>+</button>`,

  controller: `// counter.controller.ts
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => ({ count: signal(0) }),
})`,

  deck: `# deck is optional — an app framework that drives the compiler
# for a whole app: bundling, a dev loop, routing, server rendering.
deck dev --project .`,
}
