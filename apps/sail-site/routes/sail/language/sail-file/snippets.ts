// Snippets for /sail/language/sail-file — SLICES of one real component
// (apps/sail-site/src/departures-board/departures-board.sail, the board at the
// top of the page), walked part by part. Never the whole file at once.
export const code = {
  shape: `<scope>
  props: {
    title = 'Departures'
  }
</scope>

<controller>
import { controller } from '@ship/sail-runtime'
export default controller(() => ({}))
</controller>

<card>{title()}</card>`,

  template: `<card sx="p-4 w-full">
  <row align="center" justify="between" nowrap>
    <strong sx="uppercase tracking-wide text-sm">{title()}</strong>
    <span sx="text-xs text-muted-foreground font-mono">{departed()} departed</span>
  </row>
  <column gap="1" sx="mt-3 font-mono text-sm">
    {#for f of flights() key={f.id}}
      <row align="center" gap="3" nowrap>
        <span sx="grow-1 min-w-0 truncate">{f.city}</span>
        <span sx="text-muted-foreground">{f.gate}</span>
        {#match f.status}
          {:case 'boarding'} <badge tone="success">Boarding</badge>
          {:case 'delayed'}  <badge tone="destructive">Delayed</badge>
          {:default}         <badge tone="muted">On time</badge>
        {/match}
      </row>
    {/for}
  </column>
</card>`,

  scope: `<scope>
  type Flight = {
    id: string
    city: string
    gate: string
    status: 'boarding' | 'on-time' | 'delayed'
  }
  props: {
    title = 'Departures'
    seed: Flight[] = []
  }
</scope>`,

  controller: `<controller>
import { controller, signal, effect } from '@ship/sail-runtime'

export default controller((ctx) => {
  const flights = signal<Flight[]>(
    ctx.seed().length > 0 ? ctx.seed() : defaultBoard(),
  )
  const departed = signal(0)

  effect(() => {
    const t = setInterval(() => {
      flights.set((rows) => {
        const [head, ...rest] = rows
        if (head) departed.set((n) => n + 1)
        return [...rest, nextFlight()]
      })
    }, 2400)
    return () => clearInterval(t)
  })

  return { flights, departed }
})
</controller>`,

  usage: `<departures-board title="Departures — SFO"></departures-board>`,

  slotDef: `<card sx="p-4 w-full">
  ...the board...
  <slot name="footer"></slot>
</card>`,

  slotUse: `<departures-board title="Departures — SFO">
  <p slot="footer" sx="mt-3 text-xs text-muted-foreground">
    Gate changes are announced. Allegedly.
  </p>
</departures-board>`,
}
