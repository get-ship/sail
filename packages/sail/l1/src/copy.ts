/**
 * Authored sidecar — blurbs and category only.
 * Axes, props, concerns come from generated/catalog.ts.
 * Hub tile exists in docs iff a family host has copy (or we later
 * derive hub from FAMILIES and keep copy optional).
 */

export type L1Category =
  | 'inputs'
  | 'display'
  | 'feedback'
  | 'overlay'
  | 'navigation'
  | 'data'
  | 'layout'
  | 'effects'

export type L1Copy = {
  category: L1Category
  blurb: string
}

/** Family-host keyed. Parts inherit the host's copy unless listed. */
export const L1_COPY: Record<string, L1Copy> = {
  button: {
    category: 'inputs',
    blurb: 'Buttons for what people do next — save, cancel, confirm, open.',
  },
  card: {
    category: 'display',
    blurb:
      'The surface everything else sits on. variant default|outline, padding, tone, interactive, accent-bar, status-rail, fill.',
  },
  fieldset: {
    category: 'inputs',
    blurb: 'Groups related controls under a legend. Disabled freezes everything inside.',
  },
  tabs: {
    category: 'navigation',
    blurb:
      'Tabs with the keyboard suite. Host props: active · orientation · onTabChange. Strip look on <tab-list>; size on each <tab>.',
  },
}
