// Code snippets for the getting-started page. Kept OUT of the controller so the
// compiler's scope scan isn't fooled by `return {…}` / `signal(…)` patterns
// inside these strings (it reads the controller file textually).
//
// Every .sui sample is compile-checked against the in-repo sui-compiler: the
// view template compiles paired with its controller (`build` on the dir);
// the component compiles on its own. Re-run the compiler before editing a sample.
export const code = {
  install: `# the runtime + the primitive library
bun add @ship/sui-runtime @ship/sui-primitives

# the deck CLI (a native binary) compiles + serves .sui — see the deck docs`,

  template: `<!-- hello.sui — the template -->
<label>Name <input type="text" bind:value={name} /></label>
<p>Hello, {name() || 'world'}.</p>`,

  controller: `// hello.controller.ts — the logic
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => {
    const name = signal('')
    return { name }
  },
})`,

  component: `<!-- greeting.sui — a reusable component -->
<prop name="who" type="string" default="world" />

<style>
  :host { display: block; }
  p { color: var(--foreground); }
</style>

<p>Hello, {who()}.</p>`,

  run: `deck dev --project .`,
}
