import { props, tone, bool } from './props'
export const bad = props({
  tonee: tone,        // typo in the value ref
  size: sizee,        // typo — sizee not imported
  disabled: 42,       // wrong type
})
