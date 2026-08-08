<!--
  FOREIGN js-framework-benchmark chrome — not a Sail product exemplar.
  Owned at tests/perf/frameworks/keyed/sail; $BENCH_HOME keyed/sail is sync output only.
  Bootstrap class strings match the benchmark shared CSS sheet (driver layout).
  Row highlight: Action benchDanger toggles classList danger for tr.danger
  (webdriver-ts select contract). Do not copy this pattern into product Sail.
-->
<controller>
import { controller, signal, type Action } from '@ship/sail-runtime'

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

// FOREIGN: webdriver-ts scores select via tbody>tr.danger. classList toggle is
// harness-only — not a product Sail pattern.
const benchDanger: Action<boolean> = (node, on) => {
  const apply = (v: boolean) => {
    if (v) node.classList.add('danger')
    else node.classList.remove('danger')
  }
  apply(!!on)
  return {
    update: (next: boolean) => apply(!!next),
  }
}

export default controller(() => {
  const rows = signal<Row[]>([])
  // -1 = nothing selected (no row carries id -1).
  const selected = signal<number>(-1)

  return {
    rows,
    selected,
    benchDanger,

    // 01_run1k / runlots: replace the whole list.
    run: (): void => rows.set(buildData(1000)),
    runlots: (): void => rows.set(buildData(10000)),

    // 08_create1k-after1k: append without disturbing existing rows.
    add: (): void => rows.set(rows().concat(buildData(1000))),

    // 03_update10th: new object for every 10th row (same id → keyed reuse).
    update: (): void => {
      const next = rows().slice()
      for (let i = 0; i < next.length; i += 10) {
        next[i] = { ...next[i]!, label: next[i]!.label + ' !!!' }
      }
      rows.set(next)
    },

    // 09_clear.
    clear: (): void => rows.set([]),

    // 05_swap: swap rows 1 and 998 (benchmark fixed indices).
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
})
</controller>

<div class="container">
  <div class="jumbotron">
    <div class="row">
      <div class="col-md-6">
        <h1>SAIL-"keyed"</h1>
      </div>
      <div class="col-md-6">
        <div class="row">
          <div class="col-sm-6 smallpad">
            <button type="button" class="btn btn-primary btn-block" id="run" on:click={() => run()}>Create 1,000 rows</button>
          </div>
          <div class="col-sm-6 smallpad">
            <button type="button" class="btn btn-primary btn-block" id="runlots" on:click={() => runlots()}>Create 10,000 rows</button>
          </div>
          <div class="col-sm-6 smallpad">
            <button type="button" class="btn btn-primary btn-block" id="add" on:click={() => add()}>Append 1,000 rows</button>
          </div>
          <div class="col-sm-6 smallpad">
            <button type="button" class="btn btn-primary btn-block" id="update" on:click={() => update()}>Update every 10th row</button>
          </div>
          <div class="col-sm-6 smallpad">
            <button type="button" class="btn btn-primary btn-block" id="clear" on:click={() => clear()}>Clear</button>
          </div>
          <div class="col-sm-6 smallpad">
            <button type="button" class="btn btn-primary btn-block" id="swaprows" on:click={() => swaprows()}>Swap Rows</button>
          </div>
        </div>
      </div>
    </div>
  </div>
  <table class="table table-hover table-striped test-data">
    <tbody>
      <for {rows() as row} key={row.id}>
        <tr use:benchDanger={row.id === selected()}>
          <td class="col-md-1">{row.id}</td>
          <td class="col-md-4"><a on:click={() => select(row.id)}>{row.label}</a></td>
          <td class="col-md-1"><a on:click={() => remove(row.id)}><span class="glyphicon glyphicon-remove" aria-hidden="true"></span></a></td>
          <td class="col-md-6"></td>
        </tr>
      </for>
    </tbody>
  </table>
  <span class="preloadicon glyphicon glyphicon-remove" aria-hidden="true"></span>
</div>
