/**
 * Region highlight boxes for the static preview.
 *
 * Raw CSSOM paint (`cssText` / `createElement`) is banned under
 * `sail-no-imperative-style` + `sail-no-ambient-dom`. Until regions bind
 * through a Sail template (for-each boxes + sx), paint is a no-op.
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

/** No-op: overlay geometry is computed by callers; paint waits on template boxes. */
export function paintRegionOverlay(_boxes: RegionBoxUi[]): void {
  /* no-op until template-bound boxes */
}
