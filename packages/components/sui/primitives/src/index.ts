// @ship/sui-primitives — SUI primitive registry.
//
// Side-effect imports for every generic `<sui-*>` UI primitive. Each
// `<tag>.compiled.js` calls `customElements.define()` on import, so a
// single `import '@ship/sui-primitives'` at the consumer registers every
// tag in this pack globally before any view template instantiates it.
//
// Companion families (sui-tab + sui-tab-panel + sui-tabs;
// sui-accordion + sui-accordion-item; etc.) are co-located under the
// directory of the "primary" tag and listed together below.

import './sui-accordion/sui-accordion.compiled.js'
import './sui-alert-dialog/sui-alert-dialog.compiled.js'
import './sui-autocomplete/sui-autocomplete.compiled.js'
import './sui-avatar/sui-avatar.compiled.js'
import './sui-calendar/sui-calendar.compiled.js'
import './sui-card/sui-card.compiled.js'
import './sui-center/sui-center.compiled.js'
import './sui-checkbox/sui-checkbox.compiled.js'
import './sui-chip/sui-chip.compiled.js'
import './sui-grid/sui-grid.compiled.js'
import './sui-cmdk/sui-cmdk.compiled.js'
import './sui-collapsible/sui-collapsible.compiled.js'
import './sui-color-picker/sui-color-picker.compiled.js'
import './sui-combobox/sui-combobox.compiled.js'
import './sui-context-menu/sui-context-menu.compiled.js'
import './sui-date-picker/sui-date-picker.compiled.js'
import './sui-dialog/sui-dialog.compiled.js'
import './sui-diff/sui-diff.compiled.js'
import './sui-drawer/sui-drawer.compiled.js'
import './sui-dropdown-menu/sui-dropdown-menu.compiled.js'
import './sui-empty-state/sui-empty-state.compiled.js'
import './sui-error-boundary/sui-error-boundary.compiled.js'
import './sui-error-state/sui-error-state.compiled.js'
import './sui-field/sui-field.compiled.js'
import './sui-file-upload/sui-file-upload.compiled.js'
import './sui-hover-card/sui-hover-card.compiled.js'
import './sui-icon/sui-icon.compiled.js'
import './sui-input/sui-input.compiled.js'
import './sui-json-view/sui-json-view.compiled.js'
import './sui-label/sui-label.compiled.js'
import './sui-markdown/sui-markdown.compiled.js'
import './sui-nav-top/sui-nav-top.compiled.js'
import './sui-navigation-menu/sui-navigation-menu.compiled.js'
import './sui-pagination/sui-pagination.compiled.js'
import './sui-popover/sui-popover.compiled.js'
import './sui-portal/sui-portal.compiled.js'
import './sui-radio-group/sui-radio.compiled.js'
import './sui-radio-group/sui-radio-group.compiled.js'
import './sui-scroll-area/sui-scroll-area.compiled.js'
import './sui-select/sui-select.compiled.js'
import './sui-sheet/sui-sheet.compiled.js'
import './sui-skeleton/sui-skeleton.compiled.js'
import './sui-slider/sui-slider.compiled.js'
import './sui-spinner/sui-spinner.compiled.js'
import './sui-switch/sui-switch.compiled.js'
import './sui-tab-panel/sui-tab-panel.compiled.js'
import './sui-table/sui-table.compiled.js'
import './sui-tabs/sui-tabs.compiled.js'
import './sui-tabs/sui-tab.compiled.js'
import './sui-theme-picker/sui-theme-picker.compiled.js'
import './sui-timeline/sui-timeline-item.compiled.js'
import './sui-toast/sui-toast.compiled.js'
import './sui-toast/sui-toast-container.compiled.js'
import './sui-toast-host/sui-toast-host.compiled.js'
import './sui-toggle-group/sui-toggle.compiled.js'
import './sui-toggle-group/sui-toggle-group.compiled.js'
import './sui-tooltip/sui-tooltip.compiled.js'
import './sui-tree/sui-tree.compiled.js'
import './sui-tree-grid/sui-tree-grid.compiled.js'
import './sui-tree-view/sui-tree-view.compiled.js'
import './sui-tree-view/sui-tree-item.compiled.js'

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
