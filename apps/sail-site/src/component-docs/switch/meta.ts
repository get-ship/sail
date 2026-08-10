import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'switch',
  tag: 'switch',
  key: 'switchToggle',
  category: 'inputs',
  kind: 'host',
  blurb: "Native checkbox restyled as a sliding toggle (role=switch). bind:checked locks the host to a signal. size sm|md|lg scales track + thumb. tone paints the checked track only; omit tone and checked uses primary. Off track/border from component.switch.track + border; thumb always component.switch.thumb (never tone.foreground).",
  heading: "prose",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
