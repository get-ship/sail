// The source shown on the LEFT of the landing's "One program, both sides"
// panel. Kept OUT of the controller (same convention as /overview/snippets.ts)
// so the compiler's scope scan isn't fooled by the template patterns inside
// the string — and kept in hand-lockstep with the real demo in page.sail:
// the same Team program, printed on the left and running on the right.
import { signal, filteredList } from '@ship/sail-runtime'
export const code = {
  team: `<controller>
  const people = signal([ada, grace, alan])
  const { query, filtered } = filteredList(() => people())
  const invite = () => people.set((l) => [...l, next(l)])
</controller>

<column gap="3">
  <input placeholder="Filter people" bind:value={query} />
  <for {filtered() as p} key={p.id}>
    <card interactive style({ animate: enter({ opacity: 'fade' }) })>
      <avatar name={p.name} tone="primary" /> {p.name} · {p.team}
      <badge tone={p.status}>{p.status}</badge>
    </card>
  </for>
  <button on:click={invite} style({ animate: press({ scale: 'press' }) })>
    Invite</button>
</column>`,
}
