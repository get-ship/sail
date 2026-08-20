// Snippets for the component-library guide. Every Sail sample is compiled by
// the build (crates/sail/compiler/tests/site_snippets_compile.rs), and each
// one is the source of the live demo it sits next to.
export const code = {
  useThem: `<!-- components are tags. Write them like HTML. -->
<card style({ pad(4) })>
  <row gap="3" align="center">
    <avatar></avatar>
    <strong style({ grow(1).size({ min-w: 0 }) })>
    Ada Lovelace</strong>
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

<accordion>
  <strong slot="trigger">Advanced options</strong>
  <p>Hidden until asked for.</p>
</accordion>`,

  tabs: `<!-- composite components are small vocabularies: a tab is its
     title plus the panel it owns. -->
<tabs>
  <tab title="Overview">The overview panel.</tab>
  <tab title="Activity">The activity panel.</tab>
</tabs>`,

  behavior: `<!-- the behavior is already wired: this combobox filters as you
     type and commits on Enter — you wrote only the options. -->
<combobox placeholder="Pick a fruit">
  <option value="sail">Sail</option>
  <option value="svelte">Svelte</option>
  <option value="solid">Solid</option>
</combobox>`,

  icons: `<!-- name= is the lucide glyph name, kebab-case. label= makes an
     icon meaningful (role="img"); without it the icon is decoration. -->
<row gap="3" align="center">
  <icon name="search"></icon>
  <icon name="git"></icon>
  <icon name="compass"></icon>
  <icon name="zap"></icon>
  <icon name="rocket" size="lg"></icon>
  <icon name="alarm-clock" size="xl" label="Alarm"></icon>
  <icon name="refresh"></icon>
</row>`,

  iconButton: `<!-- icons inherit text color, so they follow tone for free -->
<button variant="solid" tone="primary">
  <icon name="rocket"></icon> Launch
</button>
<button variant="outline" tone="destructive">
  <icon name="trash"></icon> Delete
</button>`,

  prose: `<!-- <prose> gives rendered content typographic defaults:
     headings, paragraphs, lists, quotes, code, tables. -->
<prose>
  <h3>Release notes</h3>
  <p>The build went <strong>green</strong> at noon — see
     <code>deploy.log</code> for the timings.</p>
  <blockquote>Ship early. Ship boring.</blockquote>
  <ul>
    <li>tokens are the only styling currency</li>
    <li>elements carry their own behavior</li>
  </ul>
</prose>`,
}
