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
    const at = (ms: number, fn: () => void): void => {
      timers.push(setTimeout(fn, ms))
    }
    const t = setInterval(() => {
      step.set('write')
      at(1500, () => {
        rows.set((r) => [...r.slice(1), mk()])
        step.set('reconcile')
      })
      at(3000, () => {
        departed.set((n) => n + 1)
        step.set('paint')
      })
      at(4500, () => step.set('idle'))
    }, 6000)
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
      <span
        data-state={step() === 'paint' ? 'live' : null}
        sx="text-xs font-mono text-muted-foreground px-1 rounded-sm state-live:bg-primary state-live:text-primary-foreground"
      >{departed()} departed</span>
    </row>
    <column gap="1" sx="mt-3 font-mono text-sm">
      {#for r of rows() key={r.id}}
        <row nowrap motion:enter={{ opacity: 'fade', duration: 600 }} motion:exit={{ opacity: 'fade-out', duration: 600 }}>
          <span sx="truncate">{r.city}</span>
        </row>
      {/for}
    </column>
  </card>
  <column gap="2">
    <card group data-state={step() === 'write' ? 'live' : null} sx="p-3 opacity-75 state-live:opacity-100 state-live:bg-primary">
      <strong sx="text-sm group-state-live:text-primary-foreground">1 · the write</strong>
      <p sx="text-xs font-mono text-muted-foreground group-state-live:text-primary-foreground">the interval fires: flights.set(rows =&gt; ...)</p>
    </card>
    <card group data-state={step() === 'reconcile' ? 'live' : null} sx="p-3 opacity-75 state-live:opacity-100 state-live:bg-primary">
      <strong sx="text-sm group-state-live:text-primary-foreground">2 · the reconcile</strong>
      <p sx="text-xs font-mono text-muted-foreground group-state-live:text-primary-foreground">watch the board: one row leaves, one enters</p>
    </card>
    <card group data-state={step() === 'paint' ? 'live' : null} sx="p-3 opacity-75 state-live:opacity-100 state-live:bg-primary">
      <strong sx="text-sm group-state-live:text-primary-foreground">3 · the repaint</strong>
      <p sx="text-xs font-mono text-muted-foreground group-state-live:text-primary-foreground">one text node: the counter, top right</p>
    </card>
  </column>
</grid>
