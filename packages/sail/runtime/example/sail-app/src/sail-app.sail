<!--
  <sail-app> — single-component sanity-check demo.

  Exercises common authoring primitives in one screen so a smoke-pass
  against a fresh sail-runtime build either looks/feels right or doesn't.
  Not a benchmark, not a product — a place to click around:

    - signal / derived (counter + ticks-per-second)
    - effect-driven tick log (interval)
    - keyed <for> reorder + HTML5 DnD kanban
    - Style animate enter/exit on card add/remove
    - use:swatch + bind for dynamic column tone
-->

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
    const count = signal(0)
    const doubled = derived(() => count() * 2)

    const ticks = signal(0)
    const recentTicks = signal<Tick[]>([])
    const tickRate = signal('0')

    const tickHistory: number[] = []
    const interval = setInterval(() => {
      const n = ticks() + 1
      const now = Date.now()
      ticks.set(n)
      const ws = recentTicks().slice(-9)
      ws.push({ id: n, label: fmt(new Date(now)) })
      recentTicks.set(ws)
      tickHistory.push(now)
      while (tickHistory.length > 0 && now - tickHistory[0]! > 1000) tickHistory.shift()
      tickRate.set(tickHistory.length.toString())
    }, 250)

    const cols = signal<Column[]>(initialColumns())
    const draggingId = signal<number | null>(null)

    effect(() => {
      if (count() >= 10 && count() < 11) {
        console.log('[sail-app] count reached', count())
      }
    })

    return {
      count, doubled,
      bump: (n: number): void => count.set(count() + n),
      reset: (): void => count.set(0),

      ticks, recentTicks, tickRate,

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

<scope>
  style page = { box: space({ pad: 6, mx: auto }).size({ max-w: breakpoint-tablet }) }
  style header = { box: space({ pad-b: 4, mb: 6 }), paint: border({ edge: b, color: border, width: 1 }) }
  style muted = { paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
  style section = { box: space({ my: 6 }) }
  style sectionTitle = { box: space({ mb: 2 }), paint: ink({ color: muted-foreground }), text: type({ size: xs, weight: semibold, tracking: wide, case: uppercase }) }
  style stat = { box: space({ pad-x: 3, pad-y: 2 }), paint: fill({ color: surface-1 }).border({ color: border, width: 1 }).radius(md), text: type({ family: mono, size: sm }) }
  style tickLog = { box: space({ pad: 3 }).size({ max-h: panel-max-h-sm }).overflow({ y: auto }), paint: fill({ color: surface-1 }).border({ color: border, width: 1 }).radius(md), text: type({ family: mono, size: xs }) }
  style tickFresh = { paint: ink({ color: success }) }
  style tickWarn = { paint: ink({ color: warning }) }
  style tickMuted = { paint: ink({ color: muted-foreground }) }
  style col = { box: space({ pad: 3 }).size({ min-h: panel-min-h-daycol }), paint: fill({ color: surface-1 }).border({ color: border, width: 1 }).radius(lg) }
  style colHead = { box: space({ mb: 2 }), text: type({ size: sm, weight: semibold }) }
  style swatch = { box: size({ w: 2, h: 2 }), paint: radius(full) }
  style card = {
    box: space({ pad: 3 }),
    paint: fill({ color: background }).border({ color: border, width: 1 }).radius(md).cursor(grab),
    animate: enter({ scale: 'spring:snappy', opacity: 'fade' }).exit({ opacity: 'fade-out' }),
  }
  style cardTitle = { box: space({ mb: 1 }), text: type({ size: sm, weight: semibold }) }
  style cardMeta = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style pillHigh = { box: space({ pad-x: 1, ml: 1 }), paint: radius(full).fill({ color: destructive }).ink({ color: primary-foreground }), text: type({ family: mono, size: xs }) }
  style pillLow = { box: space({ pad-x: 1, ml: 1 }), paint: radius(full).fill({ color: muted }).ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style dropZone = { box: space({ pad: 3 }), paint: border({ color: border, width: 1 }).cursor(default).ink({ color: muted-foreground }), text: type({ size: sm, align: center }) }
</scope>

<column style(page)>
  <row align="end" justify="between" style(header)>
    <column gap="1">
      <h1>SAIL sanity-check</h1>
      <p style(muted)>standalone — no daemon, no shell, no view-host</p>
    </column>
    <row gap="2" align="center">
      <div style(stat)>ticks <strong>{ticks()}</strong></div>
      <div style(stat)>{tickRate()}/s</div>
    </row>
  </row>

  <column style(section) gap="2">
    <h2 style(sectionTitle)>Counter + derived</h2>
    <row gap="2" align="center">
      <div style(stat)>count <strong>{count()}</strong></div>
      <div style(stat)>×2 <strong>{doubled()}</strong></div>
      <button on:click={() => bump(1)}>+1</button>
      <button on:click={() => bump(-1)}>−1</button>
      <button on:click={() => bump(10)}>+10</button>
      <button on:click={() => reset()}>reset</button>
    </row>
  </column>

  <column style(section) gap="2">
    <h2 style(sectionTitle)>Live tick log</h2>
    <column style(tickLog) gap="0">
      <for {recentTicks() as row} key={row.id}>
        <if {row.id === ticks()}>
          <div style(tickFresh)>tick {row.id} @ {row.label}</div>
        {else-if {row.id % 7 === 0} =>
          <div style(tickWarn)>tick {row.id} @ {row.label}</div>
        }
        {else =>
          <div style(tickMuted)>tick {row.id} @ {row.label}</div>
        }</if>
      </for>
    </column>
  </column>

  <column style(section) gap="3">
    <h2 style(sectionTitle)>Kanban — keyed for + DnD + transitions</h2>
    <row gap="2" align="center" style({ box: space({ mb: 1 }) })>
      <div style(muted)>{totalCards()} cards</div>
      <button on:click={() => addRandom()}>+ add</button>
      <button on:click={() => chaos()}>chaos × 20</button>
      <button on:click={() => resetBoard()}>reset board</button>
    </row>
    <grid cols="3" gap="3">
      <for {columns() as col} key={col.id}>
        <column style(col) gap="2">
          <row align="center" gap="2" style(colHead)>
            <span use:swatch={col.tone} style(swatch, { paint: fill({ color: bind(swatchColor) }) })></span>
            <span>{col.title}</span>
            <span style(muted)>({col.cards.length})</span>
          </row>
          <column
            gap="2"
            on:dragover|preventDefault={(e) => onDragOver(e)}
            on:drop|preventDefault={(e) => onDrop(e, col.id, null)}
          >
            <for {col.cards as card} key={card.id}>
              <div
                draggable="true"
                on:dragstart={(e) => onDragStart(e, card.id)}
                on:dragend={onDragEnd}
                on:dragover|preventDefault|stopPropagation={(e) => onDragOver(e)}
                on:drop|preventDefault|stopPropagation={(e) => onDrop(e, col.id, card.id)}
                style(card)
              >
                <div style(cardTitle)>
                  {card.title}
                  <if {card.priority === 'high'}>
                    <span style(pillHigh)>high</span>
                  {else-if {card.priority === 'low'} => <span style(pillLow)>low</span>}</if>
                </div>
                <div style(cardMeta)>owner: {card.owner} · #{card.id}</div>
              </div>
            </for>
            <if {col.cards.length === 0}>
              <div style(dropZone)>drop a card</div>
            </if>
          </column>
        </column>
      </for>
    </grid>
  </column>
</column>
