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

  style card = { box: pad(4).w(full) }
  style titleText = { text: textCase(uppercase).tracking(wide).fontSize(sm) }
  style meta = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style list = { box: mt(3), text: fontFamily(mono).fontSize(sm) }
  style city = { text: truncate() }
  style gate = { paint: fg(muted-foreground) }
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
        <span style(city, { box: grow(1).minW(0) })>{f.city}</span>
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
