// Snippets for /sail/language/layout. Sail samples are compiled by the build;
// the diagnostic sample is real compiler output, captured verbatim.
export const code = {
  row: `<row gap="3" align="center" justify="between">
  <strong>Ada Lovelace</strong>
  <badge tone="success">Active</badge>
</row>`,

  nowrap: `<row nowrap gap="4" align="stretch">
  <navbar>…</navbar>
  <main sx="grow-1 min-w-0">…</main>
</row>`,

  column: `<column gap="2" align="start">
  <label>Email</label>
  <input type="email" placeholder="ada@example.com" />
  <button>Subscribe</button>
</column>`,

  grid: `<grid cols="3" gap="4">…</grid>

<grid min="16rem" gap="4">…</grid>`,

  spans: `<grid cols="4" gap="3">
  <card data-col-span="2">wide — spans 2</card>
  <card>one</card>
  <card>two</card>
</grid>`,

  stack: `<stack align="start" justify="end">
  <card sx="w-24 h-12 bg-muted rounded-md"></card>
  <div sx="w-4 h-4 rounded-full bg-primary"></div>
</stack>`,

  center: `<center sx="min-h-11">
  <badge tone="primary">centered</badge>
</center>`,

}
