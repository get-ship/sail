/**
 * Shared configurator chrome for docs packages.
 *
 * Layout SSOT lives in <sail-component-viewer>:
 *   slot=specimen  → left grow, centered both axes
 *   slot=knobs     → right fixed knobs column, start-aligned
 *
 * Prefer those slots over re-authoring a stage row in each host.
 *
 * Geometry atoms (for hosts still inlining the row). Width is space scale
 * (size({ w: 64 }) → --space-64), never overlay-max-w-* — those are panel-family tokens.
 */
export const STAGE_SPLIT = '{ size({ w: 100%, min-w: 0, min-h: 64 }) }'
export const STAGE_CANVAS = '{ grow(1).size({ min-w: 0, min-h: 64, basis: 0 }).pad(8) }'
export const KNOB_COLUMN =
  '{ size({ w: 64, min-h: 64 }).shrink(0).pad(5).border({ color: border-subtle, width: 1, edge: l }) }'
export const KNOB_LABEL =
  '{ fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }'
