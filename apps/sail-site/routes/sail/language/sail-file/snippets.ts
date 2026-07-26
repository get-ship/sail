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

  template: `<card sx={style.pad(4).w(full)}>
  <row align="center" justify="between" nowrap>
    <strong sx={style.textCase(uppercase).tracking(wide).fontSize(sm)}>{title()}</strong>
    <span sx={style.fontSize(xs).fg(muted-foreground).fontFamily(mono)}>{departed()} departed</span>
  </row>
  <column gap="1" sx={style.mt(3).fontFamily(mono).fontSize(sm)}>
    <for as="f" of={flights()} key={f.id}>
      <row align="center" gap="3" nowrap>
        <span grow min-w="0" sx={style.truncate()}>{f.city}</span>
        <span sx={style.fg(muted-foreground)}>{f.gate}</span>
        <match on={f.status}>
          <case of={'boarding'}> <badge tone="success">Boarding</badge>
          </case><case of={'delayed'}>  <badge tone="destructive">Delayed</badge>
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

  slotDef: `<card sx={style.pad(4).w(full)}>
  ...the board...
  <slot name="footer"></slot>
</card>`,

  slotUse: `<departures-board title="Departures — SFO">
  <p slot="footer" sx={style.mt(3).fontSize(xs).fg(muted-foreground)}>
    Gate changes are announced. Allegedly.
  </p>
</departures-board>`,
}
