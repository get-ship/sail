/**
 * Shared configurator chrome tokens for docs packages.
 * Prefer L1 layout keywords (row/column/center) + these Style atoms
 * rather than inventing a second design language per package.
 *
 * Packages may inline equivalent style() blocks — this module documents the
 * canonical stageSplit / stageCanvas / knobPanel geometry used by the legacy
 * detail host and first-class packages.
 */
export const STAGE_SPLIT = '{ box: w(full).minW(0).minH(64) }'
export const STAGE_CANVAS = '{ box: grow(1).basis(0).minW(0).minH(64).pad(8) }'
export const KNOB_PANEL =
  '{ box: w(panel-max-w-sm).shrink(0).pad(5).minH(64), paint: borderLWidth(1).borderL(border-subtle) }'
export const KNOB_LABEL =
  '{ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase) }'
