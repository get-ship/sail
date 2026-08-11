<!--
  <sail-media-csv> — CSV/TSV ViewerRenderer. Parses `src` identity or `text`
  SSR prop into createTable; renders plain <table>. Source-in, never fetch.
-->

<scope>
  props: {
    delimiter: 'auto' | 'comma' | 'tab' = 'auto'
    header = true
    text = ''
  }
</scope>
<controller>
// Pack-local relative import (not `@ship/*`) — S5.3 import law bans value
// imports from `@ship/*` in pack controllers; createTable is the primitives
// table engine, resolved via monorepo-relative path.
import { createTable, type Table } from '../../../primitives/src/sail-table/engine/create-table'
import type { ViewerSource } from '../engine/types'
import { CsvParseError, parseCsv, type CsvDelimiter } from '../engine/csv-parse'
import { emitViewerError, emitViewerLoad } from '../engine/renderer-contract'
import type { Scope } from './sail-media-csv.sail'

type CsvRecord = Record<string, string> & { id: string }

interface HeaderView {
  id: string
  label: string
}

interface RowView {
  id: string
  cells: Array<{ id: string; value: string }>
}

function textOf(src: ViewerSource | string | null): string {
  if (src == null) return ''
  if (typeof src === 'string') return src
  return src.text ?? ''
}

export default controller((_ctx: Scope) => {
    const srcSig = signal<ViewerSource | string | null>(null)
    const tableEngine = signal<Table<CsvRecord> | null>(null)
    const errorMsg = signal<string>('')

    const columns = derived<HeaderView[]>(() => {
      const t = tableEngine()
      if (!t) return []
      return t.visibleLeafColumns().map((c) => ({
        id: c.id,
        label: typeof c.columnDef.header === 'string' ? c.columnDef.header : c.id,
      }))
    })

    const rows = derived<RowView[]>(() => {
      const t = tableEngine()
      if (!t) return []
      const cols = t.visibleLeafColumns()
      return t.rows().map((r) => ({
        id: r.id,
        cells: cols.map((c) => ({
          id: `${r.id}:${c.id}`,
          value: String(r.getValue(c.id) ?? ''),
        })),
      }))
    })

    mount((host) => {
      Object.defineProperty(host, 'src', {
        configurable: true,
        get: () => srcSig(),
        set: (s: ViewerSource | string | null) => {
          srcSig.set(s)
        },
      })

      const readDelimiter = (): CsvDelimiter =>
        (host.getAttribute('delimiter') as CsvDelimiter | null) ?? 'auto'
      const readHeader = (): boolean => host.getAttribute('header') !== 'false'

      const stop = effect(() => {
        const fromSrc = textOf(srcSig())
        const text = fromSrc || (_ctx.text() ?? '')
        if (!text) {
          tableEngine.set(null)
          errorMsg.set('')
          return
        }
        try {
          const { rows, columns } = parseCsv(text, {
            header: readHeader(),
            delimiter: readDelimiter(),
          })
          const engine = createTable<CsvRecord>({
            data: rows,
            columns: columns.map((c) => ({
              id: c.field,
              accessorKey: c.field,
              header: c.label,
              enableColumnFilter: false,
            })),
            getRowId: (r) => r.id,
          })
          tableEngine.set(engine)
          errorMsg.set('')
          emitViewerLoad(host)
        } catch (e) {
          const message = e instanceof CsvParseError ? e.message : (e as Error).message
          tableEngine.set(null)
          errorMsg.set(message)
          emitViewerError(host, message)
        }
      })

      return () => {
        stop()
      }
    })

    return { srcSig, tableEngine, errorMsg, columns, rows }
  })

</controller>
<if {errorMsg()}>
  <column role="alert" gap="0" style({ box: padY(3).padX(4), paint: fg(destructive) })>
    <span style({ box: display(contents), text: fontFamily(mono).fontSize(sm) })>
      {errorMsg()}
    </span>
  </column>
{else => <table hoverable>
    <thead>
      <tr>
        <for {columns() as col} key={col.id}>
          <th>{col.label}</th>
        </for>
      </tr>
    </thead>
    <tbody>
      <for {rows() as row} key={row.id}>
        <tr data-row-id={row.id}>
          <for {row.cells as cell} key={cell.id}>
            <td>{cell.value}</td>
          </for>
        </tr>
      </for>
    </tbody>
  </table>}</if>
