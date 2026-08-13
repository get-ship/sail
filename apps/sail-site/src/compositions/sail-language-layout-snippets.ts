// Snippets for /docs/concepts/layout. Sail samples are compiled by the build;
// the diagnostic sample is real compiler output, captured verbatim.
export const code = {
  row: `<row gap="3" align="center" justify="between">
  <strong>Ada Lovelace</strong>
  <badge tone="success">Active</badge>
</row>`,

  nowrap: `<row nowrap gap="4" align="stretch">
  <navbar>…</navbar>
  <main style({ box: flow({ grow: 1 }).size({ min-w: 0 }) })>
    …
  </main>
</row>`,

  column: `<column gap="2" align="start">
  <label>Email</label>
  <input type="email" placeholder="ada@example.com" />
  <button>Subscribe</button>
</column>`,

  grid: `<grid cols="3" gap="4">…</grid>

<grid min="16rem" gap="4">…</grid>`,

  spans: `<grid cols="4" gap="3">
  <card col-span="2">wide — spans 2</card>
  <card>one</card>
  <card>two</card>
</grid>`,

  stack: `<stack align="start" justify="end">
  <card style({ box: size({ w: 24, h: 12 }), paint: fill({ color: muted }).radius(md) })>
  </card>
  <div style({ box: size(4), paint: radius(full).fill({ color: primary }) })>
  </div>
</stack>`,

  center: `<center style({ box: size({ min-h: 11 }) })>
  <badge tone="primary">centered</badge>
</center>`,

  grow: `<row gap="3" align="center">
  <strong style({ box: flow({ grow: 1 }).size({ min-w: 0 }) })>
  Ada Lovelace</strong>
  <badge tone="success" style({ box: flow({ shrink: 0 }) })>
  Active</badge>
</row>`,

  when: `<when {tablet}>
  <row nowrap>…side-by-side workbench…</row>
  {else => <column>…stacked mobile layout…</column>}
</when>`,

}
