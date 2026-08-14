// Owned snippet module. Template literals that are SAIL get compiled.
export const code = {
  page: `<!-- a page is markup -->
<column gap="4" style({ pad(6) })>
  <h1>Team</h1>
  <row gap="2">
    <input placeholder="Filter people" />
    <button variant="solid" tone="primary">Invite</button>
  </row>
  <card style({ pad(4) })>
    <row gap="3" align="center">
      <avatar></avatar>
      <strong>Ada Lovelace</strong>
      <badge tone="success">Active</badge>
    </row>
  </card>
</column>`,

  state: `<!-- state lives in a <controller> block -->
<controller>
export default controller(() => {
  const count = signal(0)
  return { count }
})
</controller>

<button on:click={() => count.set(count() + 1)}>
  Clicked {count()} times
</button>`,
}
