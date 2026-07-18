// Code snippets for the behavior page. Kept out of the controller so the
// compiler's scope scan isn't fooled by `return {…}` / `signal(…)` patterns.
// Every Sail sample is compiled by the build
// (crates/sail/compiler/tests/site_snippets_compile.rs).
export const code = {
  attach: `<!-- use: hands the element to a function. Nothing to store. -->
<textarea use:autosize></textarea>
<div use:clickOutside={close}>…</div>`,

  author: `// autofocus.ts — an action IS a function over an element.
// No registry, no base class, no lifecycle to implement.
import type { Action } from '@ship/sail-runtime'

export const autofocus: Action<void> = (node) => {
  const el = node as HTMLElement
  const id = requestAnimationFrame(() => el.focus())
  // Return the cleanup. The runtime calls it when the element goes away.
  return { destroy: () => cancelAnimationFrame(id) }
}`,

  param: `<!-- an action takes a reactive param: its update() runs on change -->
<div use:anchor={{ to: trigger, placement: 'bottom-start' }}>…</div>`,

  stdlib: `<!-- this is not a special mechanism for you: it is how the standard
     library gets ITS behavior. <combobox> is markup + use:combobox. -->
<combobox placeholder="Search">
  <option value="a">Apple</option>
</combobox>`,
}
