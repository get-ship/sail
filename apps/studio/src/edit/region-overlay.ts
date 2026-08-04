/**
 * Region highlight boxes for the static preview.
 *
 * Raw CSSOM paint (`cssText` / `createElement`) is banned under
 * `sail-no-imperative-style` + `sail-no-ambient-dom`. Until regions bind
 * through a Sail template (for-each boxes + sx), paint is a no-op.
 *
 * // SAIL-LINT-CHAT: region overlay needs Sail-template boxes (for + sx
 * // selected/hover variants) or a runtime `use:` action for geometry — not
 * // app-module createElement/cssText.
 */

export type RegionBoxUi = {
  id: string
  left: string
  top: string
  width: string
  height: string
  /** Selection (explorer / click). Takes visual precedence over hover. */
  selected: boolean
  /** Pointer hover on the static frame (may coincide with selected). */
  hover: boolean
  key: string
}

/** No-op: raw overlay paint removed (lint floor). Callers still compute boxes for labels. */
export function paintRegionOverlay(_boxes: RegionBoxUi[]): void {
  /* intentional no-op — see SAIL-LINT-CHAT above */
}
