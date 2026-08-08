/**
 * VERIFY-1 work item 1 — exhaustive layout vocabulary (matrix dimensions).
 *
 * Authoritative sources:
 *   crates/sail/layout/src/mapping.rs  (LAYOUT_RULES + SX_SCALE_RULES)
 *   crates/sail/compiler/src/check/layout.rs (diagnostics + viewports)
 *   crates/sail/layout/src/diagnostics.rs (error classes)
 *
 * This list IS the test matrix's dimensions. Any sail construct that taffy
 * cannot represent is flagged under `taffy_gaps`.
 */

/** @typedef {{ id: string, values: string[], notes?: string }} Dim */

export const VOCAB = {
  version: 1,
  sources: [
    'crates/sail/layout/src/mapping.rs',
    'crates/sail/layout/src/diagnostics.rs',
    'crates/sail/compiler/src/check/layout.rs',
  ],

  /** Layout primitives expressed as author tags → sail-* base classes. */
  primitives: /** @type {Dim} */ ({
    id: 'primitive',
    values: ['column', 'row', 'grid', 'center', 'stack'],
    notes: 'Lowered to sail-col / sail-row / sail-grid / sail-center / sail-stackz',
  }),

  participation: /** @type {Dim[]} */ ([
    {
      id: 'w',
      values: ['unset', 'full', '32', '48', '64', '1/2', '1/3'],
      notes: 'attr w= or style.w(); full→100%, fractions, scale tokens',
    },
    {
      id: 'h',
      values: ['unset', 'full', '24', '32', '48', 'screen'],
      notes: 'attr h= or style.h(); screen→100vh-ish',
    },
    {
      id: 'minW',
      values: ['unset', '0', '32'],
      notes: 'style.minW(); flex children default min-width:0 via ParentBase',
    },
    {
      id: 'minH',
      values: ['unset', '0', '24'],
    },
    {
      id: 'maxW',
      values: ['unset', '48', '64', 'full'],
    },
    {
      id: 'maxH',
      values: ['unset', '24', '32', '48'],
    },
    {
      id: 'grow',
      values: ['unset', '0', '1'],
      notes: 'style.grow(0|1) / grow attr → s-grow0/1; product is binary only',
    },
    {
      id: 'shrink',
      values: ['unset', '0', '1'],
      notes: 'style.shrink(0|1) / shrink attr',
    },
    {
      id: 'basis',
      values: ['unset'],
      notes: 'NO flex-basis atom in mapping table — taffy gap if product needs it',
    },
    {
      id: 'fill',
      values: ['unset', 'w-full', 'h-full', 'grow'],
      notes: 'fill expressed via w/h full + grow rather than a single fill atom',
    },
  ]),

  alignment: /** @type {Dim[]} */ ([
    {
      id: 'align',
      values: ['unset', 'start', 'end', 'center', 'stretch', 'baseline'],
      notes: 'align= → sail-ai-*',
    },
    {
      id: 'justify',
      values: ['unset', 'start', 'end', 'center', 'between', 'around', 'evenly'],
      notes: 'justify= → sail-jc-*',
    },
  ]),

  spacing: /** @type {Dim[]} */ ([
    {
      id: 'gap',
      values: ['unset', '0', '1', '2', '3', '4', '5', '6', '7', '8', 'density'],
      notes: 'gap= → sail-g-*; density=8px',
    },
    {
      id: 'pad',
      values: ['unset', '0', '2', '4', '6', '8'],
      notes: 'style.pad(N) → s-padN; also padX/Y/T/R/B/L/S/E scale prefixes',
    },
  ]),

  wrap_order: /** @type {Dim[]} */ ([
    {
      id: 'wrap',
      values: ['default-wrap', 'nowrap'],
      notes: 'sail-row defaults to wrap; nowrap → sail-nowrap',
    },
    {
      id: 'order',
      values: ['unset'],
      notes: 'NO order mapping in LAYOUT_RULES — taffy gap if authors need paint-order reorder',
    },
    {
      id: 'direction',
      values: ['column', 'row', 'row-reverse', 'column-reverse'],
      notes: 'via primitive tag or sail-dir-* legacy modifiers',
    },
  ]),

  overflow: /** @type {Dim} */ ({
    id: 'overflow',
    values: [
      'unset',
      'visible',
      'hidden',
      'clip',
      'auto',
      'scroll',
      'x-hidden',
      'y-hidden',
      'x-auto',
      'y-auto',
      'x-clip',
      'y-clip',
      'x-scroll',
      'y-scroll',
    ],
    notes: 'style.overflow* → s-overflow*; auto/scroll both map to taffy Scroll',
  }),

  grid: /** @type {Dim[]} */ ([
    {
      id: 'grid_cols',
      values: ['unset', '1', '2', '3', '4', '6', '12'],
      notes: 'cols=N → sail-gc-N (Fixed tracks)',
    },
    {
      id: 'grid_rows',
      values: ['unset', '1', '2', '3'],
      notes: 'rows=N → sail-gr-N',
    },
    {
      id: 'grid_min',
      values: ['unset', 'xs', 'sm', 'md', 'lg'],
      notes: 'auto-fill minmax track recipes sail-gm-*',
    },
    {
      id: 'grid_flow',
      values: ['unset', 'auto', 'dense'],
      notes: 'sail-gf-auto / sail-gf-dense',
    },
    {
      id: 'grid_tracks',
      values: ['unset', 'auto-fr-auto', 'fr-fr', 'icon-main-action'],
      notes: 'closed recipes sail-gt-*; custom var tracks → GridTemplate::Var (auto-place)',
    },
  ]),

  text: /** @type {Dim[]} */ ([
    {
      id: 'text_script',
      values: ['latin', 'cjk', 'emoji', 'mixed', 'whitespace'],
    },
    {
      id: 'text_size',
      values: ['xs', 'sm', 'base', 'md', 'lg', 'xl', '2xl', '3xl', '4xl', 'display'],
      notes: 'text-* classes; metrics table from woff2',
    },
    {
      id: 'text_family',
      values: ['body', 'mono', 'display', 'press-start'],
      notes: 'font-* classes; metrics are latin-primary with non-ASCII fallback width',
    },
    {
      id: 'text_length',
      values: ['empty', 'short', 'medium', 'long', 'wrap-stress'],
    },
  ]),

  /** Diagnostic kinds (work item 4 error classes). */
  error_classes: [
    {
      id: 'silent-clip',
      kinds: [
        'collapsed-into-overflow-hidden-parent',
        'unbounded-growth-into-overflow-hidden-parent',
      ],
      default_severity: 'Error',
      notes: 'overflow:hidden/clip parent + collapsed or overflowing content',
    },
    {
      id: 'child-overflows-parent',
      kinds: ['child-overflows-parent'],
      default_severity: 'Warning',
      notes: 'child main extent > parent; parent not silent absorber',
    },
    {
      id: 'height-chain',
      kinds: ['zero-main-axis'],
      default_severity: 'Warning',
      notes: 'zero main axis with non-zero children (height-chain collapse proxy)',
    },
  ],

  /**
   * Constructs sail expresses that taffy mapping cannot fully represent.
   * Headline findings for the verdict — not footnotes.
   */
  taffy_gaps: [
    {
      id: 'flex-basis',
      claim: 'No flex-basis atom in LAYOUT_RULES / SX_SCALE_RULES',
      impact: 'Participation is grow/shrink/size only; basis stays CSS-auto',
    },
    {
      id: 'order',
      claim: 'No CSS order / flex-order mapping',
      impact: 'Paint order follows DOM; stack uses grid-area layering instead',
    },
    {
      id: 'position-fixed-sticky',
      claim: 'style.fixed → Absolute; style.sticky → Relative in mapping',
      impact: 'Fixed/sticky viewport attachment not modeled in taffy pass',
    },
    {
      id: 'grid-var-tracks',
      claim: 'Custom-property grid tracks → GridTemplate::Var (auto-placement)',
      impact: 'Non-recipe tracks not sized like browser CSS variables',
    },
    {
      id: 'text-shaping',
      claim: 'Metrics table: latin+ASCII, no kerning/ligatures/BiDi',
      impact: 'CJK/emoji/mixed scripts use fallback glyph width',
    },
    {
      id: 'scroll-containment',
      claim: 'overflow auto/scroll both → taffy Overflow::Scroll; no contain',
      impact: 'Scroll containment / overscroll not in diagnostic box tree',
    },
    {
      id: 'unknown-custom-elements',
      claim: 'Hyphenated custom tags marked unknown_subtree; diags suppressed',
      impact: 'By design — not a mapping bug',
    },
  ],

  viewports: [
    { id: 'mobile', width: 390, height: 844 },
    { id: 'desktop', width: 1440, height: 900 },
  ],

  /** Documented geometry tolerance (px). Subpixel + font rounding. */
  geometry_tolerance_px: 2.0,
  geometry_tolerance_justification:
    'Matches PARITY-1 (PARITY_GEOM_TOL default 2). Font metric tables and browser subpixel layout routinely differ by ≤1–2px on text-driven heights; stricter 0px gates pure structure boxes only.',
}

/** Flat list of dimension ids used for per-dimension match rates. */
export function dimensionIds() {
  const ids = [VOCAB.primitives.id]
  for (const d of VOCAB.participation) ids.push(d.id)
  for (const d of VOCAB.alignment) ids.push(d.id)
  for (const d of VOCAB.spacing) ids.push(d.id)
  for (const d of VOCAB.wrap_order) ids.push(d.id)
  ids.push(VOCAB.overflow.id)
  for (const d of VOCAB.grid) ids.push(d.id)
  for (const d of VOCAB.text) ids.push(d.id)
  return ids
}

export function vocabSummary() {
  return {
    version: VOCAB.version,
    primitive_count: VOCAB.primitives.values.length,
    dimension_ids: dimensionIds(),
    error_classes: VOCAB.error_classes.map((e) => e.id),
    taffy_gaps: VOCAB.taffy_gaps.map((g) => g.id),
    tolerance_px: VOCAB.geometry_tolerance_px,
  }
}
