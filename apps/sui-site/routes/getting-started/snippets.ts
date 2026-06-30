// Code snippets for the getting-started page. Kept OUT of the controller so the
// compiler's scope scan isn't fooled by `return {…}` / `signal(…)` patterns
// inside these strings (it reads the controller file textually).
export const code = {
  install: `# add the SUI runtime to your project
npm install @ship/sui-runtime @ship/sui-dom`,
  template: `<!-- hello.sui — the template -->
<h1>Hello, {name()}</h1>
<input value={name()} on:input={(e) => name.set(e.target.value)} />`,
  controller: `// hello.controller.ts — the logic
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => {
    const name = signal('world')
    return { name }
  },
})`,
  component: `<!-- greeting.sui — a reusable component -->
<prop name="name" type="string" default="world" />

<style>
  :host { display: block; font-family: var(--font-body); }
  strong { color: var(--primary); }
</style>

<p>Hello, <strong>{name()}</strong></p>`,
}
