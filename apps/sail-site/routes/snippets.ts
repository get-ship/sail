// Code shown in the landing hero's synced panel. Kept OUT of the controller
// (same convention as /overview/snippets.ts) so the compiler's scope scan
// isn't fooled by patterns inside the string, and kept in lockstep by hand
// with the real demo in page.sail — same program, shown twice.
export const code = {
  team: `<controller>
  const people = signal([ada, grace, alan])
  const { query, filtered } = filteredList(() => people())
  const invite = () => people.set((l) => [...l, next(l)])
</controller>

<column gap="4">
  <input placeholder="Filter people" bind:value={query} />
  {#for p of filtered() key={p.id}}
    <row gap="3" align="center">
      <avatar name={p.name}></avatar>
      <strong>{p.name}</strong>
      <badge tone={p.status}>{p.status}</badge>
    </row>
  {/for}
  <button on:click={invite} motion:press={{ scale: 'press' }}>Invite</button>
</column>`,
}
