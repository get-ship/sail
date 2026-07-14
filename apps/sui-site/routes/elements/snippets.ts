// Snippets for the using-components guide. Every SUI sample is compiled by
// the build (crates/sui/compiler/tests/site_snippets_compile.rs).
export const code = {
  useThem: `<!-- components are tags. Write them like HTML. -->
<card sx="p-4">
  <row gap="3" align="center">
    <avatar></avatar>
    <strong sx="grow-1">Ada Lovelace</strong>
    <badge tone="success">Active</badge>
  </row>
</card>`,

  props: `<!-- tone picks the color from the design system;
     variant picks how much of it you get; size scales the control. -->
<button variant="solid"   tone="primary">Save</button>
<button variant="outline" tone="destructive">Delete</button>
<button variant="ghost">Cancel</button>
<badge tone="warning">rolling</badge>
<alert tone="info">Saved just now.</alert>`,

  compose: `<!-- components nest like HTML nests. Slots are named children. -->
<dialog>
  <h2>Delete this?</h2>
  <p>This cannot be undone.</p>
  <row gap="2">
    <button value="cancel">Cancel</button>
    <button value="ok" tone="destructive" variant="solid">Delete</button>
  </row>
</dialog>

<collapsible>
  <strong slot="trigger">Advanced options</strong>
  <p>Hidden until asked for.</p>
</collapsible>`,

  behavior: `<!-- the behavior is already wired: this combobox filters as you
     type and commits on Enter — you wrote only the options. -->
<combobox placeholder="Pick a framework">
  <option value="sui">SUI</option>
  <option value="svelte">Svelte</option>
  <option value="solid">Solid</option>
</combobox>`,
}
