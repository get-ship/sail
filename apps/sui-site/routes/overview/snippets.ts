// Snippets for the introduction. Every SUI sample is compiled by the build.
export const code = {
  install: `npm install @ship/sui-runtime`,

  page: `<controller>
import { controller, signal, filteredList } from '@ship/sui-runtime'

export default controller({
  scope: () => {
    const people = signal([
      { id: 1, name: 'Ada Lovelace', team: 'Engineering', status: 'active' },
      { id: 2, name: 'Grace Hopper', team: 'Engineering', status: 'away' },
    ])
    const { query, filtered } = filteredList(() => people())
    const invite = () =>
      people.set((l) => [...l, { id: l.length + 1, name: 'New member', team: 'New', status: 'active' }])
    return { query, filtered, invite }
  },
})
</controller>

<column gap="4" sx="p:6">
  <h3>Team</h3>
  <input placeholder="Filter people" bind:value={query} />

  {#for p of filtered() key={p.id}}
    <card sx="p:4" motion:enter={{ opacity: 'fade', translate-y: 'spring:snappy' }}
          motion:hover={{ scale: 'spring:gentle' }}>
      <row gap="3" align="center">
        <avatar></avatar>
        <column gap="1" sx="grow:1">
          <strong>{p.name}</strong>
          <p sx="fg:muted-foreground text:sm">{p.team}</p>
        </column>
        <badge tone={p.status === 'active' ? 'success' : 'muted'}>{p.status}</badge>
      </row>
    </card>
  {/for}

  <row gap="2">
    <button on:click={invite} variant="solid" tone="primary" motion:press={{ scale: 'press' }}>Invite</button>
    <button variant="ghost">Export</button>
  </row>
</column>`,
}
