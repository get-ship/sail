# Platform adaptor CSS — manifest

Hand-authored UA / vendor / anatomy paint the Style vocabulary cannot
express. Each file is spliced into the served stylesheet under
`/* @provenance platform/<element>.css */`.

Reason classes:

- `vendor-pseudo` — `::-webkit-*` / `::-moz-*` / `::backdrop` / meter bands
- `native-anatomy` — descendant / compound selectors on hosted native nodes
- `composite` — gradient + animation + mask that Style cannot compose
- `vocab-gap` — paint a filing would eventually grow Style to own

`gap-filed` is a named scratch note (`BUG-STDLIB-LAYER-1`,
`GAP-STDLIB-VOCAB-1`) or `none` when the class is permanent platform.

| file | element | reason | justification | gap-filed |
|---|---|---|---|---|
| accordion.css | accordion | native-anatomy | Hosted `<summary>` chevron mask + UA marker kill; sibling-stack radius merge | none |
| avatar.css | avatar | native-anatomy | Hosted `img` pack: cover for photo src; contain+pad when `icon-slug` (child Style cannot see host attr) | none |
| breadcrumb.css | breadcrumb | native-anatomy | Projected `ol`/`li`/`a` crumbs + `::before` separator | none |
| checkbox.css | checkbox | vocab-gap | `:checked` / `:indeterminate` fill + data-uri marks. Style `bg()` loses to unlayered component scopes | BUG-STDLIB-LAYER-1 |
| chip.css | chip | native-anatomy | Removable-button visibility keyed on host `removable` (child Style cannot see host attr). Inner pack is `<row cluster gap="1">` (not Style `gap()`) | none |
| code-block.css | code-block | native-anatomy | Highlighter token spans (`.tok-*`) inside the host | none |
| color.css | color | vendor-pseudo | Kill UA double-rim on `::-webkit-color-swatch` / `::-moz-color-swatch` | none |
| dialog.css | dialog | native-anatomy | `::backdrop`, side-sheet `min()` geometry, slot header/section/footer | GAP-STDLIB-VOCAB-1 |
| drawer.css | drawer | native-anatomy | Non-modal `<dialog>` edge dock + header/section/footer (same adaptor as dialog) | GAP-STDLIB-VOCAB-1 |
| file.css | file | vendor-pseudo | Hide UA Choose-file button (`::file-selector-button` / `::-webkit-file-upload-button`). Overlay input in file.sail owns click/drop | none |
| icon.css | icon | native-anatomy | Hosted child `svg` pack (width/height 100%) — glyph fill is anatomy, not a Style verb | none |
| meter.css | meter | vendor-pseudo | WebKit / Firefox meter bar + optimum bands | none |
| progress.css | progress | vendor-pseudo | WebKit / Firefox progress bar + indeterminate shimmer | GAP-STDLIB-VOCAB-1 |
| prose.css | prose | native-anatomy | Freeform descendant type scale (`h1`…`table`) — prose owns its children | none |
| radio.css | radio | vocab-gap | Nested `input:checked` radial disc. Same cascade hole as checkbox | BUG-STDLIB-LAYER-1 |
| select.css | select | vocab-gap | Chevron data-uri + `option` paint. `#888888` is allowlisted until a token exists | GAP-STDLIB-VOCAB-1 |
| skeleton.css | skeleton | composite | Shimmer gradient + `background-size` + animation | GAP-STDLIB-VOCAB-1 |
| slider.css | slider | vendor-pseudo | WebKit / Firefox range track + thumb | none |
| table.css | table | native-anatomy | Host compounds Style cannot see: `border-collapse`, dense pad, striped/hoverable on HTML `tr`, bordered cell box, sticky thead. Cell rest paint is `table/th.sail` + `td.sail` + `caption.sail` | none |
| tooltip.css | tooltip | state-channel | Visibility keys off controller `data-hidden` — motion initial-reflect stamps inline opacity, flattening class-based hides | none |

## Ledger

- **vocab-gap rows:** 6 (checkbox, radio, select, button, card, + skeleton/progress/dialog share the composite/geometry filing)
- **allowlisted value-gate hits:** select chevron `#888888`; checkbox/radio/accordion data-uri `white` / `#000` strokes

Do not add a row without a file. Do not add a file without a row.
