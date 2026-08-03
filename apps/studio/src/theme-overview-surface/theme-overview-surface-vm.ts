/**
 * Overview surface entry — re-exports session nav + composition factory.
 * Prefer createThemeSession + createOverviewEdit in new surfaces.
 */
export {
  PATH_NAV,
  OVERFLOW_NAV,
  FACET_DOORS,
  themeFacetHref,
  type ThemeFacet,
} from '../theme-session/theme-nav'
export {
  createThemeSession,
  paintIdFor,
  modesForPaintId,
  themeIdFromLocation,
  themeDisplayName,
} from '../theme-session/theme-session'
export { createOverviewEdit } from './theme-overview-edit'

import { createThemeSession } from '../theme-session/theme-session'
import { createOverviewEdit } from './theme-overview-edit'

/** Back-compat factory name used by older tests / imports. */
export function createThemeOverviewSurface() {
  const session = createThemeSession({
    facet: 'overview',
    editGuardMessage: 'Add this theme to the project to edit identity.',
  })
  const overview = createOverviewEdit(session)
  return { session, overview }
}
