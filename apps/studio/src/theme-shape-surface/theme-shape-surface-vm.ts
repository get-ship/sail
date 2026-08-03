/**
 * Shape surface entry — session + shape domain composition.
 */
export {
  ELEV_STEPS,
  RADIUS_KEYS,
  RECIPE_IDS,
  RADIUS_RAMP,
  createShapeEdit,
  mockRadiusFamily,
  withColorAlpha,
  stripColorAlpha,
  type ElevKey,
  type RadiusKey,
  type RecipeId,
} from './theme-shape-edit'
export { createThemeSession, paintIdFor, modesForPaintId } from '../theme-session/theme-session'
export { PATH_NAV, OVERFLOW_NAV } from '../theme-session/theme-nav'

import { createThemeSession } from '../theme-session/theme-session'
import { createShapeEdit } from './theme-shape-edit'

/** Back-compat factory for tests / imports. */
export function createThemeShapeSurface() {
  const session = createThemeSession({
    facet: 'shape',
    editGuardMessage: 'Add this theme to the project to edit shape.',
    saveSuccessMessage: 'Shape tokens saved to DTCG source.',
  })
  const shape = createShapeEdit(session)
  return { session, shape }
}
