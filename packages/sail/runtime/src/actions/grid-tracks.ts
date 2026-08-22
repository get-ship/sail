import type { Action } from '../action'
import { writeCssVar } from '../bind'

/**
 * `use:gridTracks={tracks}` — runtime grid-template-columns/rows string.
 *
 * The tracks here are the LIVE per-column-size track string the table
 * engine composes (e.g. `"minmax(80px, 1fr) 120px 80px"`) — a measurement,
 * not a token. The action writes it to `--sail-grid-template`; the
 * consuming `<grid tracks="--sail-grid-template">` reads it back as the
 * track string for the row it owns.
 *
 * `use:gridTracks` is the primitive — `use:style={{...}}` is the hatch.
 * Even setting a single custom property through this seal is still
 * an attempt to smuggle runtime style through a generic mechanism;
 * `use:gridTracks` names the INTENT (the grid track string lives
 * here) so the lint can target it specifically.
 */
export const gridTracks: Action<string | null | undefined> = (node, initial) => {
  const el = node as HTMLElement
  const apply = (tracks: string | null | undefined): void => {
    if (tracks == null || tracks === '') {
      writeCssVar(el, '--sail-grid-template', null)
    } else {
      writeCssVar(el, '--sail-grid-template', tracks)
    }
  }
  apply(initial)
  return {
    update: (next) => apply(next),
  }
}
