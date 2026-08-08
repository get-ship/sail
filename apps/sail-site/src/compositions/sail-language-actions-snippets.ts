// Code snippets for the behavior page. Kept out of the controller so the
// compiler's scope scan isn't fooled by `return {…}` / `signal(…)` patterns.
// Every Sail sample is compiled by the build
// (crates/sail/compiler/tests/site_snippets_compile.rs).
export const code = {
  attach: `<!-- use: hands the element to a function. Nothing to store. -->
<textarea use:autosize></textarea>
<div use:clickOutside={close}>…</div>`,

  author: `// src/actions/autofocus.ts — author hatch under the action floor.
// No style/class, no signal/store/query, no fetch, host-only.
import type { Action } from '@ship/sail-runtime'

export const autofocus: Action<void> = (node) => {
  const el = node as HTMLElement
  const id = requestAnimationFrame(() => el.focus())
  return { destroy: () => cancelAnimationFrame(id) }
}`,

  param: `<!-- an action takes a reactive param: its update() runs on change -->
<div use:anchor={{ to: trigger, placement: 'bottom-start' }}>…</div>`,

  stdlib: `<!-- Prefer the L1 widget. Widget-private behavior is closed impl —
     not a public use: you reattach. -->
<combobox placeholder="Search">
  <option value="a">Apple</option>
</combobox>
<tabs active="a">
  <tab-list><tab name="a">A</tab></tab-list>
  <tab-panel name="a">…</tab-panel>
</tabs>`,
}
