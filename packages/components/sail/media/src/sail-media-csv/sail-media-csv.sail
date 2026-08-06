<!--
  <sail-media-csv> — the CSV/TSV renderer arm of the sail-media family.

  Read-only. Parses resolved csv/tsv text (via `src` identity OR the
  `type="string"` SSR fallback prop) into `{ rows, columns }`, feeds the shared
  `createTable` engine (Backbone #1 — it does NOT re-invent a row model), and
  renders the engine's columns/rows as a plain `<table>`. The table skin is the
  framework's (contracts sheet): a plain `<table>`/`<th>`/`<td>` is already
  styled from tokens, so the renderer declares nothing.

  Density is the platform axis (`<html data-density>`): the framework's table
  skin reads the `--density-*` channel. This renderer exposes NEITHER `density`
  NOR `dense`.

  Source-in, never fetch. No `fetch`/`EventSource`/`ship.*` — bytes/text arrive
  already resolved.

  Props (build-spec §Props):
    delimiter  comma|tab|auto  reflect  — field delimiter
    header     bool, true      reflect  — first row is the header
    text       string                   — SSR/declarative fallback content

  Events (ViewerRenderer contract, via emit):
    sail-viewer-load           — first successful parse
    sail-viewer-error {message} — malformed input
-->

<scope>
  props: {
    delimiter: 'auto' | 'comma' | 'tab' = 'auto'
    header = true
    text = ''
  }
</scope>
<controller>
// sail-media-csv — the CSV/TSV renderer controller (tier-2, glue-only). Parses
// resolved text (src identity OR the `text` SSR prop) into the shared
// `createTable` engine, and exposes the engine's columns/rows as flat views the
// template renders as a plain `<table>`. Heavy parse logic lives in
// `../engine/csv-parse.ts` (the pre-committed split); the row model is the
// committed engine — this renderer owns neither.
//
// Source-in, never fetch: no `fetch`/`EventSource`/`ship.*`. Lifecycle events
// go through the sanctioned `emitViewer*` helpers (the free `emit`).
//
// Authority: build-spec Slice 4, §"Backbone #2 — ViewerRenderer contract".

import { createTable, type Table } from '@ship/sail-primitives'
import type { ViewerSource } from '../engine/types'
import { CsvParseError, parseCsv, type CsvDelimiter } from '../engine/csv-parse'
import { emitViewerError, emitViewerLoad } from '../engine/renderer-contract'
import type { Scope } from './sail-media-csv.sail'

type CsvRecord = Record<string, string> & { id: string }

/** A rendered header cell — the engine's column, flattened for the template. */
interface HeaderView {
  id: string
  label: string
}

/** A rendered row: the engine's row id + one cell per visible column. */
interface RowView {
  id: string
  cells: Array<{ id: string; value: string }>
}

/** Pull the renderable text out of a live ViewerSource (src identity). */
function textOf(src: ViewerSource | string | null): string {
  if (src == null) return ''
  if (typeof src === 'string') return src
  return src.text ?? ''
}

export default controller((_ctx: Scope) => {
    // src identity — assigned to the host `src` property in mount; the
    // table engine + error message are the reactive render inputs.
    const srcSig = signal<ViewerSource | string | null>(null)
    const tableEngine = signal<Table<CsvRecord> | null>(null)
    const errorMsg = signal<string>('')

    // The template renders the ENGINE's view of the data (its column order and
    // its row model), never the raw parse — one row model, no second grid.
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
      // src identity setter — a live JS reference, never a stringified
      // attribute. Routes into the scope signal so the parse reacts.
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

      // Re-parse whenever the src signal OR the `text` SSR prop changes.
      // The effect READS srcSig/text (reactive deps) and WRITES tableEngine/
      // errorMsg (not read here → no loop). Lifecycle emits are side effects.
      const stop = effect(() => {
        // src supersedes the `text` SSR/declarative fallback prop.
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
          // The renderer is read-only data-display: no filter/sort affordance is
          // rendered, so the engine carries the row model and nothing else.
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
  <column role="alert" gap="0">
    style.padY(3).padX(4).fg(destructive).fontFamily(mono).fontSize(sm)
{errorMsg()}</column>
<else>
  <table hoverable>
    <thead>
      <tr>
        <for {columns()} as="col" key={col.id}>
          <th>{col.label}</th>
        </for>
      </tr>
    </thead>
    <tbody>
      <for {rows()} as="row" key={row.id}>
        <tr data-row-id={row.id}>
          <for {row.cells} as="cell" key={cell.id}>
            <td>{cell.value}</td>
          </for>
        </tr>
      </for>
    </tbody>
  </table>
</else></if>
