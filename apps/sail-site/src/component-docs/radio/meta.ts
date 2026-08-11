import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'radio',
  tag: 'radio',
  key: 'radio',
  category: 'inputs',
  kind: 'host',
  blurb: "One option — native radio inside a label host. value is this option’s value; disabled is a host attribute radio-group syncs onto the nested input (input when:disabled dims the control). Label ink from component.radio.fg (--radio-foreground); control face uses free foreground/background, checked primary + RADIO_INPUT_MAP inset disc. No size/tone props. Group exclusivity, name, and onChange are radio-group composition (separate L1).",
  heading: "prose",
  host: 'doc-radio',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
