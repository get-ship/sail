<!--
  <flow-trace> — slow write → reconcile → repaint cycle for the Components chapter.
-->
<scope>
  type Row = {
    id: string
    city: string
  }

  style board = { size(max) }
  style boardTitle = { type(label) }
  // no preset: stateLive when
  style departedBadge = { fg(muted).radius(sm).type(mono-sm), when: { stateLive: bg(primary).fg(primary-foreground) } }
  style list = { type(mono) }
  style city = { type(mono) }
  style enterFade = { animate: { enter: play(fade), exit: play(fade) } }
  // no preset: stateLive when
  style stage = { border(subtle), when: { stateLive: border(strong).bg(muted) } }
  style stageTitle = { type(body-sm) }
  style stageBody = { fg(muted).type(mono-sm) }
</scope>

<controller>
const CITIES = ['Osaka', 'Nairobi', 'Tbilisi', 'Porto', 'Auckland', 'Reykjavík']

export default controller(() => {
  let seq = 0
  const mk = () => {
    const city = CITIES[seq % CITIES.length]!
    return { id: `${city}-${seq++}`, city }
  }

  const rows = signal([mk(), mk(), mk()])
  const departed = signal(0)
  const step = signal('idle')

  // Lifecycle-bound one-shots + interval; auto-cleared on dispose.
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
  <card style(board)>
    <row align="center" justify="between" nowrap>
      <strong style(boardTitle)>Board</strong>
      <span state={step() === 'paint' ? 'live' : null} style(departedBadge)>{departed()} departed</span>
    </row>
    <column gap="1" style(list)>
      <for {rows() as r} key={r.id}>
        <row nowrap style(enterFade)>
          <span style(city)>{r.city}</span>
        </row>
      </for>
    </column>
  </card>
  <column gap="2">
    <card state={step() === 'write' ? 'live' : null} style(stage)>
      <strong style(stageTitle)>1 · the write</strong>
      <p style(stageBody)>the interval fires: flights.set(rows =&gt; ...)</p>
    </card>
    <card state={step() === 'reconcile' ? 'live' : null} style(stage)>
      <strong style(stageTitle)>2 · the reconcile</strong>
      <p style(stageBody)>watch the board: one row leaves, one enters</p>
    </card>
    <card state={step() === 'paint' ? 'live' : null} style(stage)>
      <strong style(stageTitle)>3 · the repaint</strong>
      <p style(stageBody)>one text node: the counter, top right</p>
    </card>
  </column>
</grid>
