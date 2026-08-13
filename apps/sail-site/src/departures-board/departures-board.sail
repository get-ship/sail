<!--
  <departures-board> — live airport board demo for the Components chapter.
  Keyed rows keep DOM identity as the list shifts.
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

  style card = { box: space({ pad: 4 }).size({ w: full }) }
  style titleText = { text: type({ size: sm, tracking: wide, case: uppercase }) }
  style meta = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style list = { box: space({ mt: 3 }), text: type({ family: mono, size: sm }) }
  style city = { text: type({ wrap: truncate }) }
  style gate = { paint: ink({ color: muted-foreground }) }
</scope>

<controller>

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
    const pair = POOL[i % POOL.length] ?? POOL[0] ?? ['—', '—']
    const [city, gate] = pair
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

  interval(() => {
    flights.set((rows) => {
      const [head, ...rest] = rows
      if (head) departed.set((n) => n + 1)
      const promoted = rest.map((f, i) =>
        i === 0 && f.status === 'on-time' ? { ...f, status: 'boarding' as const } : f,
      )
      return [...promoted, boardRow(seq, 'on-time')]
    })
  }, 2400)

  return { flights, departed }
})
</controller>

<card style(card)>
  <row align="center" justify="between" nowrap>
    <strong style(titleText)>{title()}</strong>
    <span style(meta)>{departed()} departed</span>
  </row>
  <column gap="1" style(list)>
    <for {flights() as f} key={f.id}>
      <row align="center" gap="3" nowrap style({ animate: enter({ opacity: 'fade' }) })>
        <span style(city, { box: flow({ grow: 1 }).size({ min-w: 0 }) })>{f.city}</span>
        <span style(gate)>{f.gate}</span>
        <match {f.status}>
          {'boarding' => <badge tone="success">Boarding</badge>}
          {'delayed' => <badge tone="destructive">Delayed</badge>}
          {else => <badge tone="muted">On time</badge>}
        </match>
      </row>
    </for>
  </column>
  <slot name="footer"></slot>
</card>
