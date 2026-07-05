import { props, tone, size, bool } from './props'
// Built-ins as shorthand — how you'd really write them. A typo in a built-in:
export const p = props({
  tone,
  sizee,        // shorthand typo — key AND value are the undefined symbol
  disabled: bool,
})
