/**
 * Color surface entry — session + color domain composition.
 */
export { COLOR_ROLES, createColorEdit, type ColorRoleKey } from './theme-color-edit'
export { createThemeSession, paintIdFor, modesForPaintId } from '../theme-session/theme-session'
export { PATH_NAV } from '../theme-session/theme-nav'

import { createThemeSession } from '../theme-session/theme-session'
import { createColorEdit } from './theme-color-edit'

/** Back-compat factory for tests / imports. */
export function createThemeColorSurface() {
  const session = createThemeSession({
    facet: 'color',
    editGuardMessage: 'Add this theme to the project to edit color.',
    saveSuccessMessage: 'Color roles saved to DTCG source.',
  })
  const color = createColorEdit(session)
  return { session, color }
}
