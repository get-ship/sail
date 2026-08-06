<!--
  <flow-trace> — the Components chapter's animated tick, slowed to reading
  speed. One six-second cycle: each stage HOLDS for 1.5s while its effect
  plays out on the mini board beside it — the write (nothing moves yet; the
  write is in code), the reconcile (a row visibly leaves, one enters), the
  repaint (the counter flips and flashes). Inactive stages sit dimmed; the
  active one inverts to primary. The argument is the absent fourth card.
-->
<scope>
  type Row = {
    id: string
    city: string
  }

  const board = style.pad(4)
  const boardTitle = style.fontSize(sm).textCase(uppercase).tracking(wide)
  const departedBadge = style.fontSize(xs).fontFamily(mono).fg(muted-foreground).padX(1).radius(sm).when({
    stateLive: style.bg(primary).fg(primary-foreground),
  })
  const list = style.mt(3).fontFamily(mono).fontSize(sm)
  const city = style.truncate()
  const stage = style.pad(3).borderWidth(1).border(border).opacity(50).when({
    stateLive: style.opacity(100).border(primary).bg(muted),
  })
  const stageTitle = style.fontSize(sm)
  const stageBody = style.fontSize(xs).fontFamily(mono).fg(muted-foreground)
</scope>

<controller>
const CITIES = ['Osaka', 'Nairobi', 'Tbilisi', 'Porto', 'Auckland', 'Reykjavík']

import { debounce, interval } from '@ship/sail-runtime'

export default controller(() => {
  let seq = 0
  const mk = (): Row => {
    const city = CITIES[seq % CITIES.length]
    return { id: `${city}-${seq++}`, city }
  }

  const rows = signal<Row[]>([mk(), mk(), mk()])
  const departed = signal(0)
  const step = signal('idle')

  // Lifecycle-bound: debounce one-shots + interval, auto-cleared on dispose.
  const phaseReconcile = debounce(() => {
    rows.set((r) => [...r.slice(1), mk()])
    step.set('reconcile')
  }, 2500)
  const phasePaint = debounce(() => {
    departed.set((n) => n + 1)
    step.set('paint')
  }, 5000)
  const phaseIdle = debounce(() => step.set('idle'), 7500)
  interval(() => {
    step.set('write')
    phaseReconcile()
    phasePaint()
    phaseIdle()
  }, 10000)

  return { rows, departed, step }
})
</controller>

<grid cols="2" gap="4">
  <card>
    style(board)

    <row align="center" justify="between" nowrap>
      <strong>
        style(boardTitle)
Board</strong>
      <span
        state={step() === 'paint' ? 'live' : null}

      >
        style(departedBadge)
{departed()} departed</span>
    </row>
    <column gap="1">
      style(list)

      <for {rows()} as="r" key={r.id}>
        <row nowrap>
          style.enter({ opacity: 'fade', duration: 600 }).exit({ opacity: 'fade-out', duration: 600 })

          <span>
            style(city)
{r.city}</span>
        </row>
      </for>
    </column>
  </card>
  <column gap="2">
    <card state={step() === 'write' ? 'live' : null}>
      style(stage)

      <strong>
        style(stageTitle)
1 · the write</strong>
      <p>
        style(stageBody)
the interval fires: flights.set(rows =&gt; ...)</p>
    </card>
    <card state={step() === 'reconcile' ? 'live' : null}>
      style(stage)

      <strong>
        style(stageTitle)
2 · the reconcile</strong>
      <p>
        style(stageBody)
watch the board: one row leaves, one enters</p>
    </card>
    <card state={step() === 'paint' ? 'live' : null}>
      style(stage)

      <strong>
        style(stageTitle)
3 · the repaint</strong>
      <p>
        style(stageBody)
one text node: the counter, top right</p>
    </card>
  </column>
</grid>
