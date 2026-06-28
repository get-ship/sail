import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './sui-keyed.sui'

interface Row {
  id: number
  label: string
}

const ADJECTIVES = [
  'pretty', 'large', 'big', 'small', 'tall', 'short', 'long', 'handsome',
  'plain', 'quaint', 'clean', 'elegant', 'easy', 'angry', 'crazy', 'helpful',
  'mushy', 'odd', 'unsightly', 'adorable', 'important', 'inexpensive', 'cheap',
  'expensive', 'fancy',
]
const COLOURS = [
  'red', 'yellow', 'blue', 'green', 'pink', 'brown', 'purple', 'brown',
  'white', 'black', 'orange',
]
const NOUNS = [
  'table', 'chair', 'house', 'bbq', 'desk', 'car', 'pony', 'cookie',
  'sandwich', 'burger', 'pizza', 'mouse', 'keyboard',
]

let nextId = 1
const pick = <T>(arr: readonly T[]): T => arr[(Math.random() * arr.length) | 0]!

function buildData(count: number): Row[] {
  const data: Row[] = new Array(count)
  for (let i = 0; i < count; i++) {
    data[i] = { id: nextId++, label: `${pick(ADJECTIVES)} ${pick(COLOURS)} ${pick(NOUNS)}` }
  }
  return data
}

export default controller<Scope>({
  scope: () => {
    const rows = signal<Row[]>([])
    // -1 = nothing selected (no row carries id -1, so the `=== selected()`
    // class test is false for every row until one is clicked).
    const selected = signal<number>(-1)

    return {
      rows,
      selected,

      // 01_run1k / runlots: replace the whole list.
      run: (): void => rows.set(buildData(1000)),
      runlots: (): void => rows.set(buildData(10000)),

      // 08_create1k-after1k: append without disturbing existing rows.
      add: (): void => rows.set(rows().concat(buildData(1000))),

      // 03_update10th: new object for every 10th row (same id → keyed reuse,
      // changed value → the row's label binding re-evaluates), new array.
      update: (): void => {
        const next = rows().slice()
        for (let i = 0; i < next.length; i += 10) {
          next[i] = { ...next[i]!, label: next[i]!.label + ' !!!' }
        }
        rows.set(next)
      },

      // 09_clear.
      clear: (): void => rows.set([]),

      // 05_swap: swap rows 1 and 998 (the benchmark's fixed indices).
      swaprows: (): void => {
        const r = rows()
        if (r.length > 998) {
          const next = r.slice()
          const tmp = next[1]!
          next[1] = next[998]!
          next[998] = tmp
          rows.set(next)
        }
      },

      // 04_select: highlight the clicked row.
      select: (id: number): void => selected.set(id),

      // 06_remove: drop the clicked row.
      remove: (id: number): void => rows.set(rows().filter((r) => r.id !== id)),
    }
  },
})
