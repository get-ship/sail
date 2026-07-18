<!--
  <flow-trace> — the Components chapter's animated tick. A miniature board on
  the left; on the right, the three things that happen on every tick light up
  in order: the write, the keyed reconcile, the one text-node repaint. The
  point is what DOESN'T light up: there is no fourth step.
-->
<scope>
  type Row = {
    id: string
    city: string
  }
</scope>

<controller>
import { controller, signal, effect } from '@ship/sail-runtime'

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
    const t = setInterval(() => {
      step.set('write')
      timers.push(
        setTimeout(() => {
          rows.set((r) => [...r.slice(1), mk()])
          step.set('reconcile')
          timers.push(setTimeout(() => {
            departed.set((n) => n + 1)
            step.set('paint')
          }, 700))
          timers.push(setTimeout(() => step.set('idle'), 1500))
        }, 700),
      )
    }, 3600)
    return () => {
      clearInterval(t)
      timers.forEach((x) => clearTimeout(x))
    }
  })

  return { rows, departed, step }
})
</controller>

<grid cols="2" gap="4">
  <card sx="p-4">
    <row align="center" justify="between" nowrap>
      <strong sx="text-sm uppercase tracking-wide">Board</strong>
      <span sx="text-xs font-mono text-muted-foreground">{departed()} departed</span>
    </row>
    <column gap="1" sx="mt-3 font-mono text-sm">
      {#for r of rows() key={r.id}}
        <row nowrap motion:enter={{ opacity: 'fade' }}>
          <span sx="truncate">{r.city}</span>
        </row>
      {/for}
    </column>
  </card>
  <column gap="2">
    <card data-state={step() === 'write' ? 'live' : null} sx="p-3 state-live:bg-muted">
      <strong sx="text-sm">1 · the write</strong>
      <p sx="text-xs font-mono text-muted-foreground">flights.set(rows =&gt; ...)</p>
    </card>
    <card data-state={step() === 'reconcile' ? 'live' : null} sx="p-3 state-live:bg-muted">
      <strong sx="text-sm">2 · the reconcile</strong>
      <p sx="text-xs font-mono text-muted-foreground">keyed rows: one leaves, two shift, one enters</p>
    </card>
    <card data-state={step() === 'paint' ? 'live' : null} sx="p-3 state-live:bg-muted">
      <strong sx="text-sm">3 · the repaint</strong>
      <p sx="text-xs font-mono text-muted-foreground">one text node: the departed counter</p>
    </card>
  </column>
</grid>
