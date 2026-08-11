import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'slider',
  tag: 'slider',
  key: 'slider',
  category: 'inputs',
  kind: 'host',
  blurb: "Native single-thumb range · type=range + use:rangeFill (--rng-pct). Track/fill/border from component.slider.*; thumb face field, rim = fill. value · min · max · step · size sm|md|lg · disabled · name. No tone; no dual-thumb range.",
  heading: "prose",
  host: 'doc-slider',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
