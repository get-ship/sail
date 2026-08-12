// @ship/sail-primitives — SAIL primitive registry.
//
// Side-effect imports for every generic `<sail-*>` UI primitive. Each
// `<tag>.compiled.js` calls `customElements.define()` on import, so a
// single `import '@ship/sail-primitives'` at the consumer registers every
// tag in this package globally before any view template instantiates it.
//
// Companion families (sail-tab + sail-tab-panel + sail-tabs;
// sail-accordion + sail-accordion-item; etc.) are co-located under the
// directory of the "primary" tag and listed together below.

import './sail-calendar/sail-calendar.compiled.js'
import './sail-cmdk/sail-cmdk.compiled.js'
import './sail-color-picker/sail-color-picker.compiled.js'
import './sail-date-picker/sail-date-picker.compiled.js'
import './sail-diff/sail-diff.compiled.js'
import './sail-error-boundary/sail-error-boundary.compiled.js'
import './sail-file-upload/sail-file-upload.compiled.js'
import './sail-json-view/sail-json-view.compiled.js'
import './sail-markdown/sail-markdown.compiled.js'
import './sail-timeline/sail-timeline.compiled.js'
import './sail-timeline/sail-timeline-item.compiled.js'
import './sail-tree/sail-tree.compiled.js'
import './sail-tree-grid/sail-tree-grid.compiled.js'

// ── Canonical type unions ────────────────────────────────────────────
//
// Every primitive with a `tone` or `size` prop draws from these unions.
// Catalog hygiene: a single source of truth keeps the prop catalogs
// from drifting (e.g. one primitive ships `tone="default|primary|info"`
// while another adds `accent` and drops `muted`). Authors writing
// primitives MUST type their tone/size props from these unions; the
// audit at workbench/Primitives Audit.html flagged 5 divergences before
// this was published.

/** The canonical tone union — every Ship primitive that surfaces a
 *  semantic colour signal draws from this set. `default` is the unset
 *  baseline (foreground-on-background); `muted` is the inert grey. */
export type Tone =
  | 'default'
  | 'primary'
  | 'info'
  | 'success'
  | 'warning'
  | 'destructive'
  | 'accent'
  | 'muted'

/** The canonical size union. `md` is always the default. Primitives
 *  needing finer granularity declare additional values explicitly with
 *  rationale — sail-avatar (xs|sm|md|xl), sail-dialog (sm|md|full). */
// ── Public table engine ─────────────────────────────────────────────
//
// `createTable` is the SAIL table engine (Backbone #1) — a thin assembler
// composing the filter/sort/group/page pipeline + the orthogonal
// selection + column lanes. Re-exported here so adjacent packs (media
// at present, future `chart-table` / `sheet-table` siblings) can use
// the engine via the public package entry instead of reaching into
// `primitives/src/sail-table/engine/*` (a path that breaks `rootDir`
// isolation and the sail ⊥ sail package boundary).
export { createTable, type Table, type TableOptions } from './sail-table/engine/create-table'
// Re-export the public type catalog from `types.ts` so consumers can
// name `ColumnDef`, `Row`, `Cell`, `SortState`, etc. without reaching
// into the engine directory.
export type {
  ColumnDef,
  ColumnBase,
  AccessorColumn,
  DisplayColumn,
  GroupColumn,
  Cell,
  CellValue,
  ColumnApi,
  ColumnFilter,
  ExpandedState,
  Header,
  HeaderGroup,
  PaginationState,
  Row,
  RowSelectionState,
  RowModel,
  SortDirection,
  SortState,
  TableState,
  VisibilityState,
} from './sail-table/engine/types'
/** The canonical size union. `md` is always the default. Primitives
 *  needing finer granularity declare additional values explicitly with
 *  rationale — sail-avatar (xs|sm|md|xl), sail-dialog (sm|md|full). */
export type Size = 'sm' | 'md' | 'lg'
