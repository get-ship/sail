/**
 * Shared configurator chrome for docs packages.
 *
 * Layout SSOT lives in <sail-component-viewer>:
 *   slot=specimen  → left grow, centered both axes
 *   slot=knobs     → right fixed knobs column, start-aligned
 *
 * Prefer those slots over re-authoring a stage row in each host.
 * Hosts that still put a full row in slot=stage are legacy until migrated.
 *
 * Geometry atoms (for hosts still inlining the row). Width is space scale
 * (size({ w: 64 }) → --space-64), never panel-max-w-* — those are panel-family tokens.
 */
export const STAGE_SPLIT = '{ box: size({ w: full, min-w: 0, min-h: 64 }) }'
export const STAGE_CANVAS = '{ box: flow({ grow: 1 }).size({min-w: 0, min-h: 64, basis: 0}).space({ pad: 8 }) }'
export const KNOB_COLUMN =
  '{ box: size({ w: 64, min-h: 64 }).flow({ shrink: 0 }).space({ pad: 5 }), paint: border({ color: border-subtle, width: 1, edge: l }) }'
export const KNOB_LABEL =
  '{ paint: ink({ color: muted-foreground }), text: type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }'
