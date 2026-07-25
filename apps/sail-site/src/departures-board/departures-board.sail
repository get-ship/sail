<!--
  <departures-board> — the component the "Components" chapter builds, slice by
  slice. A live airport board: every few seconds the boarding flight departs,
  the next flight starts boarding, and a new city rolls onto the bottom of the
  board. Keyed rows keep DOM identity as the list shifts.
-->
<scope>
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

  const card = style.pad(4).w(full)
  const titleText = style.textCase(uppercase).tracking(wide).fontSize(sm)
  const meta = style.fontSize(xs).fg(muted-foreground).fontFamily(mono)
  const list = style.mt(3).fontFamily(mono).fontSize(sm)
  const city = style.grow(1).minW(0).truncate()
  const gate = style.fg(muted-foreground)
</scope>

<controller>
import { controller, signal, effect } from '@ship/sail-runtime'

const POOL: Array<[string, string]> = [
  ['Reykjavík', 'A2'],
  ['Osaka', 'B7'],
  ['Nairobi', 'C1'],
  ['Valparaíso', 'A5'],
  ['Tbilisi', 'B3'],
  ['Auckland', 'C4'],
  ['Porto', 'A9'],
  ['Ulaanbaatar', 'B1'],
]

export default controller((ctx) => {
  let seq = 0
  const boardRow = (i: number, status: Flight['status']): Flight => {
    const [city, gate] = POOL[i % POOL.length]
    return { id: `${city}-${seq++}`, city, gate, status }
  }

  const flights = signal<Flight[]>(
    ctx.seed().length > 0
      ? ctx.seed()
      : [
          boardRow(0, 'boarding'),
          boardRow(1, 'on-time'),
          boardRow(2, 'delayed'),
          boardRow(3, 'on-time'),
          boardRow(4, 'on-time'),
        ],
  )
  const departed = signal(0)

  effect(() => {
    const t = setInterval(() => {
      flights.set((rows) => {
        const [head, ...rest] = rows
        if (head) departed.set((n) => n + 1)
        const promoted = rest.map((f, i) =>
          i === 0 && f.status === 'on-time' ? { ...f, status: 'boarding' as const } : f,
        )
        return [...promoted, boardRow(seq, 'on-time')]
      })
    }, 2400)
    return () => clearInterval(t)
  })

  return { flights, departed }
})
</controller>

<card sx={card}>
  <row align="center" justify="between" nowrap>
    <strong sx={titleText}>{title()}</strong>
    <span sx={meta}>{departed()} departed</span>
  </row>
  <column gap="1" sx={list}>
    {#for f of flights() key={f.id}}
      <row align="center" gap="3" nowrap motion:enter={{ opacity: 'fade' }}>
        <span sx={city}>{f.city}</span>
        <span sx={gate}>{f.gate}</span>
        {#match f.status}
          {:case 'boarding'} <badge tone="success">Boarding</badge>
          {:case 'delayed'}  <badge tone="destructive">Delayed</badge>
          {:default}         <badge tone="muted">On time</badge>
        {/match}
      </row>
    {/for}
  </column>
  <slot name="footer"></slot>
</card>
