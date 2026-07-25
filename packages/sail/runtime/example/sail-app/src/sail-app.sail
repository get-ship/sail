<!--
  <sail-app> — single-component sanity-check demo.

  Goal: exercise the common authoring primitives in one screen so a
  smoke-pass against a fresh sail-runtime build either looks/feels
  right or doesn't. Not a benchmark, not a product — just a place
  to click around.

  What's here:
    - signal/derived: counter at the top, ticks-per-second derived
    - effect: 250ms tick keeps a sparkline-ish log moving
    - bindFor + keyed reorder: kanban-ish board with HTML5 DnD
    - data-*: per-card priority + selected highlight
    - on: handlers, including async one (chrome.modal not used —
      standalone build has no daemon)
    - motion: enter/exit on card add/remove (fade + scale)
    - update() callback exercise: the chaos button mutates the row
      list, which forces bindFor through the diff path the per-field
      gating + monotone fast-path optimize.
-->

<scope>
  props: {
    shadow = false
  }
</scope>
<controller>
import { controller, signal, derived, effect } from '@ship/sail-runtime'

type Priority = 'low' | 'normal' | 'high'
interface Card {
  id: number
  title: string
  owner: string
  priority: Priority
  tone: string
}
interface Column {
  id: string
  title: string
  tone: string
  cards: Card[]
}

const OWNERS = ['ada', 'beck', 'cora', 'dax', 'ezra']
const TITLES = [
  'wire sse reconnect', 'profile bindFor at 10k', 'audit focus ring',
  'tokenize router segments', 'split kanban controller', 'add gauge component',
  'sweep aria-labels', 'centralize date format', 'replace findByTag',
]
const PRIORITIES: Priority[] = ['low', 'normal', 'normal', 'normal', 'high']
const TONES = ['var(--info)', 'var(--success)', 'var(--warning)', 'var(--accent)', 'var(--primary)']

let nextId = 0
const pick = <T>(arr: readonly T[]): T => arr[Math.floor(Math.random() * arr.length)]
const mkCard = (): Card => ({
  id: ++nextId,
  title: pick(TITLES),
  owner: pick(OWNERS),
  priority: pick(PRIORITIES),
  tone: pick(TONES),
})

const initialColumns = (): Column[] => [
  { id: 'todo',  title: 'To do',       tone: 'var(--info)',    cards: Array.from({ length: 4 }, mkCard) },
  { id: 'doing', title: 'In progress', tone: 'var(--warning)', cards: Array.from({ length: 3 }, mkCard) },
  { id: 'done',  title: 'Done',        tone: 'var(--success)', cards: Array.from({ length: 2 }, mkCard) },
]

const moveBetween = (
  state: Column[],
  cardId: number,
  targetCol: string,
  beforeId: number | null,
): Column[] => {
  const next = state.map((c) => ({ ...c, cards: [...c.cards] }))
  let moving: Card | null = null
  for (const c of next) {
    const idx = c.cards.findIndex((x) => x.id === cardId)
    if (idx >= 0) { moving = c.cards.splice(idx, 1)[0]; break }
  }
  if (!moving) return state
  const target = next.find((c) => c.id === targetCol)
  if (!target) return state
  if (beforeId == null) target.cards.push(moving)
  else {
    const idx = target.cards.findIndex((c) => c.id === beforeId)
    target.cards.splice(idx < 0 ? target.cards.length : idx, 0, moving)
  }
  return next
}

interface Tick { id: number; label: string }
const fmt = (d: Date): string =>
  `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}:${String(d.getSeconds()).padStart(2, '0')}.${String(d.getMilliseconds()).padStart(3, '0')}`

export default controller(() => {
    // Counter + derived.
    const count = signal(0)
    const doubled = derived(() => count() * 2)

    // Ticks driven by a 250ms interval; recentTicks is a sliding window
    // of the last 10. effect demonstrates a side-effect (DOM-free) that
    // updates a signal — common pattern for streaming sources.
    const ticks = signal(0)
    const lastTickWall = signal<number>(Date.now())
    const recentTicks = signal<Tick[]>([])
    const tickRate = signal('0')

    const tickHistory: number[] = []
    const interval = setInterval(() => {
      const n = ticks() + 1
      const now = Date.now()
      ticks.set(n)
      lastTickWall.set(now)
      const ws = recentTicks().slice(-9)
      ws.push({ id: n, label: fmt(new Date(now)) })
      recentTicks.set(ws)
      // Rolling 1-second tick rate.
      tickHistory.push(now)
      while (tickHistory.length > 0 && now - tickHistory[0]! > 1000) tickHistory.shift()
      tickRate.set(tickHistory.length.toString())
    }, 250)

    // Kanban state.
    const cols = signal<Column[]>(initialColumns())
    const draggingId = signal<number | null>(null)

    // Demo effect — log to console when count crosses 10. Just to
    // show effect() is alive; harmless otherwise.
    effect(() => {
      if (count() >= 10 && count() < 11) {
        console.log('[sail-app] count reached', count())
      }
    })

    return {
      // counter
      count, doubled,
      bump: (n: number): void => count.set(count() + n),
      reset: (): void => count.set(0),

      // ticks
      ticks, recentTicks, tickRate,

      // kanban
      cols, draggingId,
      totalCards: derived(() => cols().reduce((n, c) => n + c.cards.length, 0)),
      columns: derived(() => cols()),

      addRandom: (): void => {
        const c = cols()
        const col = pick(c)
        cols.set(c.map((x) => x.id === col.id ? { ...x, cards: [...x.cards, mkCard()] } : x))
      },
      chaos: (): void => {
        let state = cols()
        for (let i = 0; i < 20; i++) {
          const op = Math.random()
          if (op < 0.45 && state.some((x) => x.cards.length > 0)) {
            const filled = state.filter((x) => x.cards.length > 0)
            const from = pick(filled)
            const card = pick(from.cards)
            const to = pick(state)
            const beforeId = to.cards.length > 0 ? pick(to.cards).id : null
            state = moveBetween(state, card.id, to.id, beforeId)
          } else if (op < 0.8) {
            const col = pick(state)
            state = state.map((x) => x.id === col.id ? { ...x, cards: [...x.cards, mkCard()] } : x)
          } else {
            const all = state.flatMap((x) => x.cards)
            if (all.length === 0) continue
            const victim = pick(all)
            state = state.map((col) => ({ ...col, cards: col.cards.filter((c) => c.id !== victim.id) }))
          }
        }
        cols.set(state)
      },
      resetBoard: (): void => { nextId = 0; cols.set(initialColumns()) },

      onDragStart: (e: DragEvent, id: number): void => {
        draggingId.set(id)
        if (e.dataTransfer) {
          e.dataTransfer.setData('text/plain', String(id))
          e.dataTransfer.effectAllowed = 'move'
        }
      },
      onDragEnd: (): void => draggingId.set(null),
      onDragOver: (e: DragEvent): void => {
        if (e.dataTransfer) e.dataTransfer.dropEffect = 'move'
      },
      onDrop: (e: DragEvent, col: string, beforeId: number | null): void => {
        const raw = e.dataTransfer?.getData('text/plain') ?? ''
        const id = raw ? Number(raw) : draggingId()
        if (id == null || id === beforeId) { draggingId.set(null); return }
        cols.set(moveBetween(cols(), id, col, beforeId))
        draggingId.set(null)
      },

      __interval: interval,
    }
  })
</controller>

<div>
  <header>
    <div>
      <h1>SAIL sanity-check</h1>
      <p>standalone — no daemon, no shell, no view-host</p>
    </div>
    <div data-row="true">
      <div data-counter="true">ticks <strong>{ticks()}</strong></div>
      <div data-counter="true">{tickRate()}/s</div>
    </div>
  </header>

  <section>
    <h2>Counter + derived</h2>
    <div data-row="true">
      <div data-counter="true">count <strong>{count()}</strong></div>
      <div data-counter="true">×2 <strong>{doubled()}</strong></div>
      <button on:click={() => bump(1)}>+1</button>
      <button on:click={() => bump(-1)}>−1</button>
      <button on:click={() => bump(10)}>+10</button>
      <button on:click={() => reset()}>reset</button>
    </div>
  </section>

  <section>
    <h2>Live tick log (effect re-fires each tick)</h2>
    <div>
      {#for row of recentTicks() key={row.id}}
        <div data-ticks-row="true" aria-current={row.id === ticks() ? 'fresh' : (row.id % 7 === 0 ? 'warn' : 'false')}>
          tick {row.id} @ {row.label}
        </div>
      {/for}
    </div>
  </section>

  <section>
    <h2>Kanban — bindFor + DnD + transitions</h2>
    <div data-row="true" sx={style.mb(3)}>
      <div>{totalCards()} cards</div>
      <button on:click={() => addRandom()}>+ add</button>
      <button on:click={() => chaos()}>chaos × 20</button>
      <button on:click={() => resetBoard()}>reset board</button>
    </div>
    <div>
      {#for col of columns() key={col.id}}
        <section data-col={col.id}>
          <div>
            <span data-tone="plain" use:swatch={col.tone} sx={style.bg(bind(swatchColor))}></span>
            <span>{col.title}</span>
            <span>({col.cards.length})</span>
          </div>
          <ol data-col-cards={col.id}
              on:dragover|preventDefault={(e) => onDragOver(e)}
              on:drop|preventDefault={(e) => onDrop(e, col.id, null)}>
            {#for card of col.cards key={card.id}}
              <li data-card="true"
                  data-card={card.id}
                  draggable="true"
                  motion:enter={{ scale: 'spring:snappy', opacity: 'fade' }}
                  motion:exit={{ opacity: 'fade-out' }}
                  on:dragstart={(e) => onDragStart(e, card.id)}
                  on:dragend={onDragEnd}
                  on:dragover|preventDefault|stopPropagation={(e) => onDragOver(e)}
                  on:drop|preventDefault|stopPropagation={(e) => onDrop(e, col.id, card.id)}>
                <div>
                  {card.title}
                  {#if card.priority === 'high'}
                    <span>high</span>
                  {:else if card.priority === 'low'}
                    <span>low</span>
                  {/if}
                </div>
                <div>owner: {card.owner} · #{card.id}</div>
              </li>
            {/for}
            {#if col.cards.length === 0}
              <li data-card="true" style="border-style:dashed;cursor:default;text-align:center;color:var(--muted)">
                drop a card
              </li>
            {/if}
          </ol>
        </section>
      {/for}
    </div>
  </section>
</div>
