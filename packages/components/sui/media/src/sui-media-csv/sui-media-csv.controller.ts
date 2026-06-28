// sui-media-csv — the CSV/TSV renderer controller (tier-2, glue-only). Parses
// resolved text (prop:src identity OR the `text` SSR prop) and composes the
// committed `sui-table` via a `createTable` engine bound through `prop:table`.
// Heavy parse logic lives in `../engine/csv-parse.ts` (the pre-committed split).
//
// Source-in, never fetch: no `fetch`/`EventSource`/`ship.*`. Lifecycle events
// go through the sanctioned `emitViewer*` helpers (the free `emit`).
//
// Authority: build-spec Slice 4, §"Backbone #2 — ViewerRenderer contract".

import { controller, effect, signal } from '@ship/sui-runtime'
import { createTable, type Table } from '@ship/sui-primitives'
import type { ViewerSource } from '../engine/types'
import { CsvParseError, parseCsv, type CsvDelimiter } from '../engine/csv-parse'
import { emitViewerError, emitViewerLoad } from '../engine/renderer-contract'
import type { Scope } from './sui-media-csv.sui'

type CsvRecord = Record<string, string> & { id: string }

/** Pull the renderable text out of a live ViewerSource (prop:src identity). */
function textOf(src: ViewerSource | string | null): string {
  if (src == null) return ''
  if (typeof src === 'string') return src
  return src.text ?? ''
}

export default controller({
  scope: (_ctx: Scope) => {
    // prop:src identity — assigned to the host `src` property in mount; the
    // table engine + error message are the reactive render inputs.
    const srcSig = signal<ViewerSource | string | null>(null)
    const tableEngine = signal<Table<CsvRecord> | null>(null)
    const errorMsg = signal<string>('')
    return { srcSig, tableEngine, errorMsg }
  },

  mount(state, host) {
    // prop:src identity setter — a live JS reference, never a stringified
    // attribute. Routes into the scope signal so the parse reacts.
    Object.defineProperty(host, 'src', {
      configurable: true,
      get: () => state.srcSig,
      set: (s: ViewerSource | string | null) => {
        state.srcSig = s
      },
    })

    const readDelimiter = (): CsvDelimiter =>
      (host.getAttribute('delimiter') as CsvDelimiter | null) ?? 'auto'
    const readHeader = (): boolean => host.getAttribute('header') !== 'false'

    // Re-parse whenever the prop:src signal OR the `text` SSR prop changes.
    // The effect READS srcSig/text (reactive deps) and WRITES tableEngine/
    // errorMsg (not read here → no loop). Lifecycle emits are side effects.
    const stop = effect(() => {
      // prop:src supersedes the `text` SSR/declarative fallback prop.
      const fromSrc = textOf(state.$signals.srcSig())
      const text = fromSrc || (state.$signals.text() ?? '')
      if (!text) {
        state.tableEngine = null
        state.errorMsg = ''
        return
      }
      try {
        const { rows, columns } = parseCsv(text, {
          header: readHeader(),
          delimiter: readDelimiter(),
        })
        const engine = createTable<CsvRecord>({
          data: rows,
          columns: columns.map((c) => ({ id: c.field, accessorKey: c.field, header: c.label })),
          getRowId: (r) => r.id,
        })
        state.tableEngine = engine
        state.errorMsg = ''
        emitViewerLoad(host)
      } catch (e) {
        const message = e instanceof CsvParseError ? e.message : (e as Error).message
        state.tableEngine = null
        state.errorMsg = message
        emitViewerError(host, message)
      }
    })

    return () => {
      stop()
    }
  },
})
