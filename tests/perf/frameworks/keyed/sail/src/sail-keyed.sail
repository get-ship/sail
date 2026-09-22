<!--
  js-framework-benchmark keyed implementation.
  Driver class names (Bootstrap + tr.danger) are applied AFTER emit by a
  script that lives only in the benchmark checkout.
-->
<controller>
import { controller, signal } from '@ship/sail-runtime'

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

export default controller(() => {
  const rows = signal<Row[]>([])
  const selected = signal<number>(-1)

  return {
    rows,
    selected,
    run: (): void => rows.set(buildData(1000)),
    runlots: (): void => rows.set(buildData(10000)),
    add: (): void => rows.set(rows().concat(buildData(1000))),
    update: (): void => {
      const next = rows().slice()
      for (let i = 0; i < next.length; i += 10) {
        next[i] = { ...next[i]!, label: next[i]!.label + ' !!!' }
      }
      rows.set(next)
    },
    clear: (): void => rows.set([]),
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
    select: (id: number): void => selected.set(id),
    remove: (id: number): void => rows.set(rows().filter((r) => r.id !== id)),
  }
})
</controller>

<column>
  <row>
    <column>
      <h1>SAIL-"keyed"</h1>
    </column>
    <column>
      <row>
        <column>
          <button type="button" id="run" on:click={() => run()}>Create 1,000 rows</button>
        </column>
        <column>
          <button type="button" id="runlots" on:click={() => runlots()}>Create 10,000 rows</button>
        </column>
        <column>
          <button type="button" id="add" on:click={() => add()}>Append 1,000 rows</button>
        </column>
        <column>
          <button type="button" id="update" on:click={() => update()}>Update every 10th row</button>
        </column>
        <column>
          <button type="button" id="clear" on:click={() => clear()}>Clear</button>
        </column>
        <column>
          <button type="button" id="swaprows" on:click={() => swaprows()}>Swap Rows</button>
        </column>
      </row>
    </column>
  </row>
  <table>
    <tbody>
      <for {rows() as row} key={row.id}>
        <tr aria-selected={row.id === selected()}>
          <td>{row.id}</td>
          <td><a on:click={() => select(row.id)}>{row.label}</a></td>
          <td><a on:click={() => remove(row.id)}>×</a></td>
          <td></td>
        </tr>
      </for>
    </tbody>
  </table>
</column>
