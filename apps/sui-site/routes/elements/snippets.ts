// Code snippets for the elements page. Kept out of the controller so the
// compiler's scope scan isn't fooled by `return {…}` / `signal(…)` patterns.
// Every SUI sample is compiled by the build
// (crates/sui/compiler/tests/site_snippets_compile.rs).
export const code = {
  native: `<!-- a dialog is <dialog>. The browser already has one. -->
<button popovertarget="menu">Open</button>
<dialog id="confirm">
  <h2>Delete this?</h2>
  <button value="cancel">Cancel</button>
  <button value="ok" tone="destructive">Delete</button>
</dialog>`,

  language: `<!-- <combobox> is a language element: it compiles to markup + a
     use: action, and the custom element disappears. -->
<combobox placeholder="Pick a fruit">
  <option value="apple">Apple</option>
  <option value="banana">Banana</option>
</combobox>`,

  variant: `<!-- a variant is a PROP, not a component. A sheet is a dialog
     that comes in from the side; an alert is a dialog with a role. -->
<dialog side="end">…</dialog>
<dialog role="alertdialog">…</dialog>`,

  layout: `<!-- the axis IS the name. No flex-direction to get backwards. -->
<row gap="3" align="center">…</row>
<column gap="2">…</column>
<grid cols="3" gap="4">…</grid>`,
}
