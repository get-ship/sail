// Snippets for /sail/language/sail-file — SLICES of one real component
// (apps/sail-site/src/departures-board/departures-board.sail, the board at the
// top of the page), walked part by part. Never the whole file at once.
export const code = {
  // Only the template is required. Scope is optional; a controller is only
  // needed when the component has state or handlers — never an empty one.
  shape: `<scope>
  props: {
    title = 'Departures'
  }
</scope>

<card>{title()}</card>`,

  template: `<card>
    style({ box: pad(4).w(full) })
  <row align="center" justify="between" nowrap>
    <strong>
      style({ text: textCase(uppercase).tracking(wide).fontSize(sm) })
      {title()}</strong>
    <span>
      style({ paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) })
      {departed()} departed</span>
  </row>
  <column gap="1">
    style({ box: mt(3) })
    <for {flights()} as="f" key={f.id}>
      <row align="center" gap="3" nowrap>
        <span>
          style({ box: grow(1).minW(0), text: truncate().fontFamily(mono).fontSize(sm) })
          {f.city}</span>
        <span>
          style({ paint: fg(muted-foreground), text: fontFamily(mono).fontSize(sm) })
          {f.gate}</span>
        <match {f.status}>
          <case {'boarding'}> <badge tone="success">Boarding</badge>
          </case><case {'delayed'}>  <badge tone="destructive">Delayed</badge>
          </case><default>         <badge tone="muted">On time</badge>
        </default></match>
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

  slotDef: `<card>
    style({ box: pad(4).w(full) })
  ...the board...
  <slot name="footer"></slot>
</card>`,

  slotUse: `<departures-board title="Departures — SFO">
  <p slot="footer">
    style({ box: mt(3), paint: fg(muted-foreground), text: fontSize(xs) })
    Gate changes are announced. Allegedly.
  </p>
</departures-board>`,
}
