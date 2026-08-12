import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'meter',
  tag: 'meter',
  key: 'meter',
  category: 'feedback',
  kind: 'host',
  blurb: "Native <meter> gauge — a measurement in range, not task progress. value/min/max set the reading; low/high/optimum drive platform fill bands painted from component.meter.track / optimum / suboptimum / sub-suboptimum. size scales bar height. No tone prop.",
  heading: "prose",
  host: 'doc-meter',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
