# @ship/sui-primitives — dissolution audit

Audit of every component and export in `packages/components/sui/primitives`
against the L1 standard library (`crates/sui/compiler/stdlib/*.sui`, 40
elements, plus the native+skin tags: `button`, `input`, `select`, `textarea`,
`dialog`, `details`, `table`, …). Rule applied: **primitives dissolves into
other packs or is deleted in the event of duplication.** This document is the
plan only — no dissolution has been executed.

Method: classified each `src/` entry by (a) presence in `src/index.ts` (the
pack registers exactly 13 tags; everything else is unregistered residue),
(b) an L1 equivalent in the stdlib registry, and (c) real usage — grepped
`apps/`, `packages/`, `crates/`, `tests/` for tag usage (`<sui-*>`) and module
imports (`@ship/sui-primitives`).

## Current state (evidence)

- `src/index.ts` registers **13 custom elements** and re-exports the table
  engine (`createTable` + types) and the `Tone`/`Size` unions.
- **24 component directories are orphans**: a `.controller.ts` + generated
  `.sui.d.ts` with **no `.sui` source** — leftovers of the L1 cutover commits
  (`cutover(l1): …`, `refactor(sui): execute the surface decision — 25
  components deleted`). None is imported by `index.ts`; none is imported by
  anything else in the repo.
- **The pack's test suite is entirely red: 44/44 test files fail** before
  collecting a single test — `test/setup.ts` imports `../src/sui-icon/registry`,
  deleted when `<icon>` moved to L1. Nothing has depended on these tests
  passing for some time.
- Several orphan tests only resolve through **stale `.sail-out` mirror
  artifacts** of deleted `.sui` sources (e.g. `sui-drawer-a11y.test.ts`,
  `sui-app-icon.test.ts`); `sui-sheet.controller.ts` imports
  `./engine/create-sheet` and `./view/geometry`, directories that no longer
  exist.
- Zero `<sui-*>` tags from this pack appear in any template outside the pack.
  The docs site registers nothing (`apps/sui-site/src/register-all.ts`: "the
  docs are built from L1"); its `package.json` dependency is vestigial.
- The one **real** external consumer is `@ship/sui-media`
  (`sui-media-csv.controller.ts` imports `createTable`/`Table`).

## Classification table

Status: LIVE = registered via `index.ts`; ORPHAN = controller + `.sui.d.ts`
residue, no `.sui`, unregistered, unreferenced.

| Component / export | Status | Classification | Disposition | Rationale |
| --- | --- | --- | --- | --- |
| `sui-alert-dialog` | ORPHAN | Duplicate of L1 | Delete | Cut over to native `<dialog>` + `useDialog` (commit 5e1c5e1); residue only. |
| `sui-autocomplete` | ORPHAN | Duplicate of L1 `<combobox>` | Delete | Filter-and-commit picker is exactly the L1 combobox. |
| `sui-calendar` (+ `engine/`) | LIVE | Novel | Move → new **@ship/sui-inputs** | Month-grid + range engine; no L1 equivalent; pairs with date-picker. |
| `sui-cmdk` | LIVE | Novel | Move → new **@ship/sui-cmdk** | Command palette over native `<dialog>` + `useDialog`; generic, self-contained, no L1 overlap. |
| `sui-color-picker` (+ `color-math.ts`) | LIVE | Novel | Move → new **@ship/sui-inputs** | Rich input the platform's `<input type=color>` can't theme; no L1 equivalent. |
| `sui-combobox` | ORPHAN | Duplicate of L1 `<combobox>` | Delete | The L1 element exists and is the one home. |
| `sui-context-menu` | ORPHAN | Duplicate of L1 `<context-menu>` | Delete | Same. |
| `sui-date-picker` | LIVE | Novel | Move → new **@ship/sui-inputs** | ISO date field + popover month grid; no L1 equivalent; shares the calendar domain. |
| `sui-dialog` | ORPHAN | Duplicate of L1 native `<dialog>` skin | Delete | Explicitly dropped in the alert-dialog cutover. |
| `sui-diff` | LIVE | Novel | Move → new **@ship/sui-prose** | Unified-diff renderer; L1 `<code-block>` renders code, not diffs. |
| `sui-drawer` | ORPHAN | Duplicate of L1 dialog/sheet | Delete | Side-panel overlay covered by the L1 dialog/sheet cutover (5609a85). |
| `sui-dropdown-menu` | ORPHAN | Duplicate of L1 `<dropdown>` + `<menu-item>` | Delete | Same pattern, one home. |
| `sui-error-boundary` | LIVE | **Duplicate of L1 `<error-boundary>`** | Delete | The only live duplicate. L1 element + runtime `attachBoundary` own fault isolation. Migration note below (event rename). |
| `sui-file-upload` | LIVE | Novel | Move → new **@ship/sui-inputs** | Drag-drop upload zone + file list; no L1 equivalent. |
| `sui-hover-card` | ORPHAN | Duplicate of L1 `<hover-card>` | Delete | Residue. |
| `sui-icon` | ORPHAN | Duplicate of L1 `<icon>` | Delete | Cut over; its deleted `registry.ts` is what breaks the whole test suite today. |
| `sui-input` | ORPHAN | Duplicate of L1 native `<input>` + `<field>` | Delete | Native+skin plus the L1 field stack cover it. |
| `sui-json-view` | LIVE | Novel | Move → new **@ship/sui-data** | Collapsible JSON tree; data-display sibling of tree/tree-grid. |
| `sui-markdown` | LIVE | Novel | Move → new **@ship/sui-prose** | Dependency-free markdown renderer via `bindHtml`; future consumer: sui-site's hand-rolled `sui-prose`. |
| `sui-navigation-menu` (+ `sui-navigation-item`) | ORPHAN | Duplicate of L1 `<navbar>` + `<menu-item>` | Delete | Superseded by the navbar work (df630e5). |
| `sui-nav-top` | ORPHAN | Duplicate of L1 `<navbar>` | Delete | "`<nav-top>` is now `<navbar>`" — same commit. |
| `sui-pagination` | ORPHAN | Duplicate of L1 `<pagination>` | Delete | Residue. |
| `sui-popover` | ORPHAN | Duplicate of L1 `<popover>` | Delete | Residue. |
| `sui-portal` | ORPHAN | Dead | Delete | Native top layer (`[popover]`, `<dialog>`) removed the need for a portal element; no L1 equivalent because none is needed. |
| `sui-scroll-area` | ORPHAN | Duplicate of L1 `<scroll-area>` | Delete | Residue. |
| `sui-select` | ORPHAN | Duplicate of L1 native `<select>` skin | Delete | Covered by the native+skin cutover (5609a85). |
| `sui-sheet` | ORPHAN | Dead | Delete | Spreadsheet experiment; its `engine/` + `view/` are already deleted — the controller imports files that don't exist. If a spreadsheet is ever wanted it's a new pack built fresh. |
| `sui-table` — `engine/` (`createTable` + types) | LIVE (module export) | Novel, headless | Move → new **@ship/sui-data** | The one export with a real external consumer (`@ship/sui-media` CSV table). Not an element; a headless engine. |
| `sui-table` — `view/` (wiring, light-dom-css, data-events) | residue | Dead | Delete | Referenced by nothing, not even tests. |
| `sui-tab-panel` | ORPHAN | Duplicate of L1 `<tab-panel>` | Delete | Residue. |
| `sui-tabs` (+ `sui-tab`) | ORPHAN | Duplicate of L1 `<tabs>`/`<tab-list>`/`<tab>` | Delete | Residue. |
| `sui-timeline` (+ `sui-timeline-item`) | LIVE | Novel | **Promote to L1 candidate** (else → @ship/sui-prose) | Static, tone-riding, slot-composed display list — exactly the L1 element shape (like `<alert>`/`<badge>`); near-zero JS. Owner's call. |
| `sui-toast` (+ `sui-toast-container`) | ORPHAN | Duplicate of L1 `<toast>` + `<toast-region>` | Delete | Residue. |
| `sui-toast-host` | ORPHAN | Duplicate of L1 `<toast-region>` | Delete | Second toast residue with the same successor. |
| `sui-toggle-group` (+ `sui-toggle`) | ORPHAN | Duplicate of L1 `<toggle-group>`/`<toggle>` | Delete | Residue. |
| `sui-tree` | LIVE | Novel | Move → new **@ship/sui-data** | Virtualized tree over `createTable` (getSubRows+expanded) + `createVirtualizer`; no L1 equivalent. |
| `sui-tree-grid` | LIVE | Novel | Move → new **@ship/sui-data** | Tree × columns over the same engine; travels with tree + table engine. |
| `sui-tree-view` (+ `sui-tree-item`) | ORPHAN | Dead | Delete | Superseded in-pack by `sui-tree`. **Keep the `TreeNode` type** — fold it into `sui-tree` (9 files import it from here). |
| `Tone` / `Size` unions (`index.ts`) | LIVE (types) | Dead | Delete | Zero importers; the tone/size contract is enforced by the compiler + `contracts.css` now. If a typed home is wanted, `@ship/tokens` — not a component pack. |
| `overlay-focus.ts` | helper | Dead | Delete | Only importers are the dead `sui-dialog`/`sui-drawer` controllers. |
| `test-utils.ts` | helper | Dead with the pack | Delete | Pack-internal mount helpers; live components' tests move with their components onto `@ship/test-setup`'s `renderComponent` (the sui-mermaid pattern). |
| `fast-formula-parser` (devDep) | dep | Dead | Delete | Belonged to the deleted `sui-sheet` engine. |

**Tally — 37 component directories: 22 duplicates of L1 (delete), 12 novel
(move to 4 new packs; 1 of the 12 is an L1-promotion candidate), 3 dead
(delete).** Plus the root exports: table engine moves, `Tone`/`Size` unions
and both helpers die with the pack.

## End state

`@ship/sui-primitives` is **deleted**. Its 12 survivors regroup by domain
(names are placeholders — owner picks):

| New pack | Contents | Notes |
| --- | --- | --- |
| **@ship/sui-data** | `sui-table/engine` (`createTable` + type catalog), `sui-tree`, `sui-tree-grid`, `sui-json-view`, `TreeNode` type | The headless engine plus everything built on it. `@ship/sui-media` switches its import here. |
| **@ship/sui-inputs** | `sui-calendar` (+ engine), `sui-date-picker`, `sui-color-picker`, `sui-file-upload` | Rich form inputs beyond native+skin. |
| **@ship/sui-prose** | `sui-markdown`, `sui-diff` (and `sui-timeline` if not promoted to L1) | Rendered-text tiles; sui-site's `sui-prose` is a natural future consumer. |
| **@ship/sui-cmdk** | `sui-cmdk` | Self-contained palette; could fold into sui-inputs if four packs is too many. |

### Consumer migrations required at execution time

- `@ship/sui-media` — `import { createTable, type Table } from '@ship/sui-primitives'`
  (controller + `.sui`) → `@ship/sui-data`.
- `@ship/sui-viewer` — tests side-effect-import `@ship/sui-primitives` to
  register tags their comments say are "sui-error-boundary / sui-alert /
  sui-code"; the templates now use L1 elements, so the imports just drop.
  **Latent bug found while auditing:** `sui-render` mounts L1
  `<error-boundary>` but listens for the legacy primitives event
  `sui-error-boundary:caught`; the L1 boundary emits `sui-error-caught`
  (`runtime/src/actions/error-boundary.ts`), so the viewer's runtime-throw
  path never fires today. Fix the event name regardless of dissolution.
- `apps/sui-site` — drop the vestigial `@ship/sui-primitives` dependency
  (`register-all.ts` already registers nothing); regenerate
  `components.gen.ts`; tutorial lesson `08-sui-primitives`
  (`packages/sui/tutorial/src/prose/08-sui-primitives/`) rewrites against the
  new packs.
- `packages/sui/test-setup` — `component-leak-gate.test.ts` hard-codes the
  primitives package dir as its specimen corpus; retarget to a surviving pack.
  `lucide-icons.ts` doc comment references the pack's deleted icon cache.
- `crates/deck` — `@ship/sui-primitives` appears only as an example string in
  bundler/config test fixtures; rename opportunistically, nothing breaks.

### Suggested execution order

1. Carve **@ship/sui-data** out first and switch `@ship/sui-media` — the only
   runtime consumer; everything else is then unreferenced.
2. Fix the viewer event-name bug + drop its vestigial imports.
3. Create sui-inputs / sui-prose / sui-cmdk from the live sources (each pack
   mirrors the @ship/sui-mermaid shape: `.sui` with inline controller,
   `renderComponent`-based tests, `suiComponentVitestConfig`). Port each
   component's dedicated tests; leave the dead orphan tests behind.
4. Decide `sui-timeline`: L1 stdlib promotion or sui-prose.
5. Delete `packages/components/sui/primitives` and the stale
   `.sail-out/packages/components/sui/primitives` mirror; drop the sui-site
   dep; rewrite tutorial lesson 08; retarget the leak gate.

### Open questions for the owner

- **Pack granularity** — 4 new packs vs folding cmdk into sui-inputs (3).
- **`sui-timeline`** — L1 promotion (it fits the static-element bar) or pack.
- **`Tone`/`Size` unions** — delete outright (recommended; zero importers) or
  re-home in `@ship/tokens` as the typed mirror of the tone contract.
- **Naming** — sui-data / sui-inputs / sui-prose / sui-cmdk are placeholders.
