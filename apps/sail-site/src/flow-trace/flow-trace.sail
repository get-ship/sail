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

export default controller(() => {
  let seq = 0
  const mk = (): Row => {
    const city = CITIES[seq % CITIES.length]
    return { id: `${city}-${seq++}`, city }
  }

  const rows = signal<Row[]>([mk(), mk(), mk()])
  const departed = signal(0)
  const step = signal('idle')

  effect(() => {
    const timers: ReturnType<typeof setTimeout>[] = []
    const at = (ms: number, fn: () => void): void => {
      timers.push(setTimeout(fn, ms))
    }
    const t = setInterval(() => {
      step.set('write')
      at(2500, () => {
        rows.set((r) => [...r.slice(1), mk()])
        step.set('reconcile')
      })
      at(5000, () => {
        departed.set((n) => n + 1)
        step.set('paint')
      })
      at(7500, () => step.set('idle'))
    }, 10000)
    return () => {
      clearInterval(t)
      timers.forEach((x) => clearTimeout(x))
    }
  })

  return { rows, departed, step }
})
</controller>

<grid cols="2" gap="4">
  <card sx={board}>
    <row align="center" justify="between" nowrap>
      <strong sx={boardTitle}>Board</strong>
      <span
        data-state={step() === 'paint' ? 'live' : null}
        sx={departedBadge}
      >{departed()} departed</span>
    </row>
    <column gap="1" sx={list}>
      {#for r of rows() key={r.id}}
        <row nowrap fx:enter={{ opacity: 'fade', duration: 600 }} fx:exit={{ opacity: 'fade-out', duration: 600 }}>
          <span sx={city}>{r.city}</span>
        </row>
      {/for}
    </column>
  </card>
  <column gap="2">
    <card data-state={step() === 'write' ? 'live' : null} sx={stage}>
      <strong sx={stageTitle}>1 · the write</strong>
      <p sx={stageBody}>the interval fires: flights.set(rows =&gt; ...)</p>
    </card>
    <card data-state={step() === 'reconcile' ? 'live' : null} sx={stage}>
      <strong sx={stageTitle}>2 · the reconcile</strong>
      <p sx={stageBody}>watch the board: one row leaves, one enters</p>
    </card>
    <card data-state={step() === 'paint' ? 'live' : null} sx={stage}>
      <strong sx={stageTitle}>3 · the repaint</strong>
      <p sx={stageBody}>one text node: the counter, top right</p>
    </card>
  </column>
</grid>
