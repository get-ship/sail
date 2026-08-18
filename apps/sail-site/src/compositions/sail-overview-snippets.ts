// Code snippets for the introduction. The `page` snippet IS the live demo's
// program; the tour snippets are deliberately tiny. All compiled by the build.
import { signal, derived, effect, filteredList, applyTheme } from '@ship/sail-runtime'
import { query } from '@ship/sail-runtime/query'
export const code = {
  // Section 2 — the whole demo, six lines. The page renders these same lines live.
  hello: `const playing = signal(true)

<column gap="2">
  <demo-player playing={playing()}></demo-player>
  <row gap="2" align="center">
    <switch bind:checked={playing}></switch>
    <badge>{playing() ? 'playing' : 'paused'}</badge>
  </row>
</column>`,
  // The strict config, pure JSON (rendered by <sail-json-view> on the page).
  configJson: `{"preset": "strict", "raw_containers": true}`,
  // REAL config files from this repository, verbatim — never invent these.
  configStrict: `// apps/sail-site/routes/sail.config.json — this site's docs pages
{
  "preset": "strict",
  "raw_containers": true
}`,
  configComponent: `// packages/components/sail/code-editor/sail.config.json — embeds CodeMirror
{ "raw_containers": true, "stylesheets": [] }

// packages/components/sail/canvas/sail.config.json — may fetch from self, nowhere else
{ "raw_containers": true,
  "stylesheets": [], "network": ["self"] }`,
  page: `<controller>
export default controller(() => {
  const people = signal([
    { id: 1, name: 'Ada Lovelace', status: 'active' },
    { id: 2, name: 'Grace Hopper', status: 'away' },
  ])
  const { query, filtered } = filteredList(() => people())
  const invite = () =>
    people.set((l) => [...l, { id: l.length + 1, name: 'New member', status: 'active' }])
  return { query, filtered, invite }
})
</controller>

<input placeholder="Filter people" bind:value={query} />
<for {filtered() as p} key={p.id}>
  <card style({ pad(4), animate: { enter: play(slide-up) } })>
    <row gap="3" align="center">
      <avatar></avatar>
      <strong style({ grow(1) })>
        {p.name}
      </strong>
      <badge tone={p.status === 'active' ? 'success' : 'muted'}>{p.status}</badge>
    </row>
  </card>
</for>
<button on:click={invite} variant="solid" tone="primary">Invite</button>`,

  layout: `<row gap="3" align="center">…</row>     <!-- x axis -->
<column gap="4">…</column>              <!-- y axis -->
<grid cols="3" gap="3">…</grid>         <!-- 2D -->`,

  components: `<button variant="solid" tone="primary">Save</button>
<button variant="outline" tone="destructive">Delete</button>
<badge tone="success">Active</badge>
<alert tone="info">Saved.</alert>`,

  styling: `<card style({ pad(5).bg(surface-1).radius(lg).elevation(sm), when: { hover: elevation(md) } })>
  <p style({ fg(muted-foreground).type({ size: sm }) })>
    On the system, or it does not compile.</p>
</card>`,

  state: `const count = signal(0)
const doubled = derived(() => count() * 2)
effect(() => console.log(count()))   // re-runs on change`,

  data: `const users = query(() => api.users.list())
// users() is { kind: 'loading' } | { kind: 'ok', value } | { kind: 'error', error }`,

  fx: `<card style({ animate: { enter: play(slide-up) } })>
    …</card>`,

  theming: `
applyTheme('console')   // one attribute; every component follows`,
}
