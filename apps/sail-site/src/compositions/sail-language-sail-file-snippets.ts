// Snippets for /docs/concepts/sail-file — SLICES of one real component
// (apps/sail-site/src/departures-board/departures-board.sail, the board at the
// top of the page), walked part by part. Never the whole file at once.
import { signal, interval } from '@ship/sail-runtime'
export const code = {
  // Only the template is required. Scope is optional; a controller is only
  // needed when the component has state or handlers — never an empty one.
  shape: `<scope>
  props: {
    title = 'Departures'
  }
</scope>

<card>{title()}</card>`,

  template: `<card style({ box: space({ pad: 4 }).size({ w: full }) })>
  <row align="center" justify="between" nowrap>
    <strong style({ text: type({ size: sm, tracking: wide, case: uppercase }) })>
      {title()}</strong>
    <span style({ paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) })>
      {departed()} departed</span>
  </row>
  <column gap="1" style({ box: space({ mt: 3 }) })>
    <for {flights() as f} key={f.id}>
      <row align="center" gap="3" nowrap>
        <span style({ box: flow({ grow: 1 }).size({ min-w: 0 }), text: type({ wrap: truncate, family: mono, size: sm }) })>
          {f.city}</span>
        <span style({ paint: ink({ color: muted-foreground }), text: type({ family: mono, size: sm }) })>
          {f.gate}</span>
        <match {f.status}>
          {'boarding' => <badge tone="success">Boarding</badge>}{'delayed' => <badge tone="destructive">Delayed</badge>}{else => <badge tone="muted">On time</badge>}</match>
      </row>
    </for>
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

  // Ambient runtime idents (controller, signal, effect, …) need no import —
  // the compiler injects them for the idents the block actually references.
  controller: `<controller>
export default controller((ctx) => {
  const flights = signal<Flight[]>(
    ctx.seed().length > 0 ? ctx.seed() : defaultBoard(),
  )
  const departed = signal(0)

  interval(() => {
    flights.set((rows) => {
      const [head, ...rest] = rows
      if (head) departed.set((n) => n + 1)
      return [...rest, nextFlight()]
    })
  }, 2400)

  return { flights, departed }
})
</controller>`,

  usage: `<departures-board title="Departures — SFO"></departures-board>`,

  slotDef: `<card style({ box: space({ pad: 4 }).size({ w: full }) })>
  ...the board...
  <slot name="footer"></slot>
</card>`,

  slotUse: `<departures-board title="Departures — SFO">
  <p slot="footer" style({ box: space({ mt: 3 }), paint: ink({ color: muted-foreground }), text: type({ size: xs }) })>
    Gate changes are announced. Allegedly.
  </p>
</departures-board>`,
}
