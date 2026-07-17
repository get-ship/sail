// @ship/sail-primitives — SUI primitive registry.
//
// Side-effect imports for every generic `<sui-*>` UI primitive. Each
// `<tag>.compiled.js` calls `customElements.define()` on import, so a
// single `import '@ship/sail-primitives'` at the consumer registers every
// tag in this pack globally before any view template instantiates it.
//
// Companion families (sui-tab + sui-tab-panel + sui-tabs;
// sui-accordion + sui-accordion-item; etc.) are co-located under the
// directory of the "primary" tag and listed together below.

import './sui-calendar/sui-calendar.compiled.js'
import './sui-cmdk/sui-cmdk.compiled.js'
import './sui-color-picker/sui-color-picker.compiled.js'
import './sui-date-picker/sui-date-picker.compiled.js'
import './sui-diff/sui-diff.compiled.js'
import './sui-error-boundary/sui-error-boundary.compiled.js'
import './sui-file-upload/sui-file-upload.compiled.js'
import './sui-json-view/sui-json-view.compiled.js'
import './sui-markdown/sui-markdown.compiled.js'
import './sui-timeline/sui-timeline.compiled.js'
import './sui-timeline/sui-timeline-item.compiled.js'
import './sui-tree/sui-tree.compiled.js'
import './sui-tree-grid/sui-tree-grid.compiled.js'

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
 *  rationale — sui-avatar (xs|sm|md|xl), sui-dialog (sm|md|full). */
// ── Public table engine ─────────────────────────────────────────────
//
// `createTable` is the SUI table engine (Backbone #1) — a thin assembler
// composing the filter/sort/group/page pipeline + the orthogonal
// selection + column lanes. Re-exported here so adjacent packs (media
// at present, future `chart-table` / `sheet-table` siblings) can use
// the engine via the public package entry instead of reaching into
// `primitives/src/sui-table/engine/*` (a path that breaks `rootDir`
// isolation and the sui ⊥ sui pack boundary).
export { createTable, type Table, type TableOptions } from './sui-table/engine/create-table'
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
} from './sui-table/engine/types'
/** The canonical size union. `md` is always the default. Primitives
 *  needing finer granularity declare additional values explicitly with
 *  rationale — sui-avatar (xs|sm|md|xl), sui-dialog (sm|md|full). */
export type Size = 'sm' | 'md' | 'lg'
