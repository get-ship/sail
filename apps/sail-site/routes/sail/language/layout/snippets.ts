// Snippets for /sail/language/layout. SAIL samples are compiled by the build;
// the diagnostic sample is real compiler output, captured verbatim.
export const code = {
  axes: `<row gap="3">…</row>        <!-- x: left to right -->
<column gap="3">…</column>  <!-- y: top to bottom -->
<stack>…</stack>            <!-- z: one on top of another -->
<grid cols="3" gap="4">…</grid>
<center>…</center>`,

  layoutSxError: `error: [sail-sx] sx does not do layout — the parent decides how
  its children sit. Write:

      <row align="center" justify="between">

  A layout element names the axis and closes the vocabulary, which is what
  stops two authors diverging on four flex properties.`,

  row: `<!-- gap · align (cross axis) · justify (main axis) -->
<row gap="3" align="center" justify="between">
  <strong>Ada Lovelace</strong>
  <badge tone="success">Active</badge>
</row>`,

  nowrap: `<!-- a row WRAPS by default: it is a flow, not a clamp.
     When the design depends on one line, say so: -->
<row nowrap gap="4" align="stretch">
  <navbar>…</navbar>
  <main sx="grow-1 min-w-0">…</main>
</row>`,

  column: `<!-- gap · align -->
<column gap="2" align="start">
  <label>Email</label>
  <input type="email" />
</column>`,

  grid: `<!-- a fixed count… -->
<grid cols="3" gap="4">…</grid>

<!-- …or a responsive track: fill as many columns of at least min as fit -->
<grid min="16rem" gap="4">…</grid>`,

  spans: `<grid cols="4" gap="3">
  <card data-col-span="2">wide</card>
  <card>one</card>
  <card>two</card>
</grid>`,

  nesting: `<!-- rows of columns of rows. That is every layout you have ever built. -->
<row gap="4" align="start">
  <column gap="2">
    <h3>Details</h3>
    <p>…</p>
  </column>
  <column gap="2">
    <h3>Activity</h3>
    <p>…</p>
  </column>
</row>`,
}
