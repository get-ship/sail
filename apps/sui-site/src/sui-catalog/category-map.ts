// Pack taxonomy for the public SUI component catalog (apps/sui-site
// /components page).
//
// ONE source of truth for two questions:
//   1. Which surface a public tag belongs to: the `primitives` pack
//      ("Standard Library") vs any Sail-authored pack ("Packs").
//   2. Which tags are NOT public catalog entries — the internal sub-primitives
//      that compose but never appear standalone on /components
//      (`HIDDEN_TAGS`).
//
// Why the flat `PACK` map lives here:
//   - The mapping is a `Record<tag, packName>` — a static lookup table keyed
//     by tag (the compiler-built CEM produces the same set of string keys).
//   - The pack name on each entry IS the `package` field the deck build stamps
//     on every CEM module (`@ship/sui-charts`, `@ship/sui-canvas`, …); a
//     reviewer can flip a tag from one pack to another by editing one line.
//   - Tags whose CEM `package` is the standard library (`@ship/sui-primitives`)
//     are NOT in this map — they fall through to the `Surface = 'stdlib'`
//     bucket by construction. Re-classifying a primitive as a pack requires
//     moving it to a different package, which is the desired friction.

/** The two top-level surfaces surfaced as a single chip row on /components.
 *  `stdlib` is the default; `pack` groups every Sail-authored pack. */
export type Surface = 'stdlib' | 'pack'

// ── Pack display order ──────────────────────────────────────────────────────

/** Canonical display order for packs (the chip row's group headers on the
 *  Packs surface, AND the pack-name headers rendered above each card group). */
export const PACK_ORDER: readonly string[] = [
  'Charts',
  'Canvas',
  'Chat Elements',
  'Media',
  'Code Editor',
  'Mermaid',
] as const

/** Display label per pack key. Keys equal `PACK_ORDER` entries; values are
 *  the same human-readable names (no normalization needed — the keys ARE the
 *  labels). Kept as a record so a header render can fall back to the raw key
 *  if a pack disappears from PACK_ORDER. */
export const PACK_LABELS: Record<string, string> = {
  'Charts': 'Charts',
  'Canvas': 'Canvas',
  'Chat Elements': 'Chat Elements',
  'Media': 'Media',
  'Code Editor': 'Code Editor',
  'Mermaid': 'Mermaid',
}

// ── Hidden sub-primitives ───────────────────────────────────────────────────

/** Tags that compose a parent primitive, are pure scaffolding, or need a
 *  non-attribute prop seam (the JS-prop path) to render meaningfully — none
 *  of them belong in a public gallery cell. The composed parent (e.g.
 *  `<sui-accordion-item>` rolls up into `<sui-accordion>`,
 *  `<sui-tab-panel>` into `<sui-tabs>`) is the public API. Review/edit
 *  this list in one place. */
export const HIDDEN_TAGS: Record<string, true> = {
  // accordion — the item is internal; the parent is the API
  'sui-accordion-item': true,
  // tabs — tab + panel are slots of `<sui-tabs>`
  'sui-tab': true,
  'sui-tab-panel': true,
  // tree — item is a slot of `<sui-tree>`
  'sui-tree-item': true,
  // timeline — item is a slot of `<sui-timeline>`
  'sui-timeline-item': true,
  // toast family — toast + toast-container + toast-host internals; the
  // composed parent routes a `ship:notify` event into the container's
  // programmatic `show()` API. The cell preview cannot drive that event
  // from a bare mount, so the cell would always be empty. Keep the host
  // entry visible — its description explains the contract.
  'sui-toast': true,
  'sui-toast-container': true,

  // radio (radio-group is the public entry; radio is its slot)
  'sui-radio': true,

  // Brand mark — the SHIP logo, not a public component. It is brand
  // chrome (it composes the topbar, account menu, etc.) and has no
  // place in the /components catalog card grid.
  // Viewer scaffolding — these primitives need the JS-prop path
  // (`prop:source` / `prop:compile` set as host JS properties, NOT a
  // `setAttribute` write) to render. A bare mount leaves them empty,
  // which is misleading in a public gallery. The full demo harness lives
  // on the /components/[tag] detail page.
  'sui-render': true,
  'sui-source-panel': true,
  'sui-prop-knobs': true,
  'sui-prop-matrix': true,
  'sui-component-viewer': true,
}

// ── Public tag → pack mapping ───────────────────────────────────────────────

/** Pack assignment for every public pack tag. Organized by pack for review;
 *  the runtime only cares about the flat Record. Tags NOT in this map are
 *  standard-library primitives (`@ship/sui-primitives`). The labels are the
 *  exact `PACK_LABELS` keys (and `PACK_ORDER` entries) so a single string
 *  drives display, grouping, and chip rendering — no second source of truth. */
export const PACK: Record<string, string> = {
  // ── @ship/sui-charts ────────────────────────────────────────────────
  'sui-area-chart':      'Charts',
  'sui-bar-chart':       'Charts',
  'sui-donut':           'Charts',
  'sui-gauge':           'Charts',
  'sui-heatmap':         'Charts',
  'sui-line-chart':      'Charts',
  'sui-sparkline':       'Charts',

  // ── @ship/sui-canvas ────────────────────────────────────────────────
  // The annotation family (sui-annotation-{layer,list,marker}) lives under
  // Canvas alongside sui-board / sui-canvas / sui-draw / sui-graph-canvas.
  // Each is its own standalone tag — the .sui files declare them as
  // distinct custom elements — so the gallery surfaces all of them.
  'sui-annotation-layer':  'Canvas',
  'sui-annotation-list':   'Canvas',
  'sui-annotation-marker': 'Canvas',
  'sui-board':             'Canvas',
  'sui-canvas':            'Canvas',
  'sui-draw':              'Canvas',
  'sui-graph-canvas':      'Canvas',

  // ── @ship/sui-chat-elements ─────────────────────────────────────────
  'sui-citation':        'Chat Elements',
  'sui-composer':        'Chat Elements',
  'sui-message':         'Chat Elements',
  'sui-message-list':    'Chat Elements',
  'sui-thinking-dots':   'Chat Elements',
  'sui-tool-call':       'Chat Elements',

  // ── @ship/sui-media ─────────────────────────────────────────────────
  'sui-media-av':        'Media',
  'sui-media-csv':       'Media',
  'sui-media-image':     'Media',

  // ── @ship/sui-code-editor ───────────────────────────────────────────
  'sui-code-editor':     'Code Editor',

  // ── @ship/sui-mermaid ───────────────────────────────────────────────
  'sui-mermaid':         'Mermaid',
}

// ── Lookup helpers ──────────────────────────────────────────────────────────

/** Surface for a public tag — `pack` when the tag is in `PACK`, otherwise
 *  `stdlib` (the @ship/sui-primitives pack). */
export function surfaceOf(tag: string): Surface {
  return PACK[tag] ? 'pack' : 'stdlib'
}

/** Pack label for a pack tag, `null` for standard-library tags. */
export function packOf(tag: string): string | null {
  return PACK[tag] ?? null
}