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
 * (w(64) → --space-64), never panel-max-w-* — those are panel-family tokens.
 */
export const STAGE_SPLIT = '{ box: w(full).minW(0).minH(64) }'
export const STAGE_CANVAS = '{ box: grow(1).basis(0).minW(0).minH(64).pad(8) }'
export const KNOB_COLUMN =
  '{ box: w(64).shrink(0).pad(5).minH(64), paint: borderLWidth(1).borderL(border-subtle) }'
export const KNOB_LABEL =
  '{ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase) }'
