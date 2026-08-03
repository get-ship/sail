/**
 * Themes path / IA constants — single source for facet chrome + doors.
 */
import { appRouteBase, hrefForActivity } from '../shell/shell-page-vm'

export type ThemeFacet =
  | 'overview'
  | 'color'
  | 'shape'
  | 'assets'
  | 'components'

/** Path / facet nav — crumb menu + doors. */
export const PATH_NAV = [
  { key: 'overview', value: 'overview', icon: 'layout-dashboard', label: 'Overview' },
  { key: 'color', value: 'color', icon: 'palette', label: 'Color' },
  { key: 'shape', value: 'shape', icon: 'shapes', label: 'Shape' },
  { key: 'assets', value: 'assets', icon: 'sparkles', label: 'Assets' },
  { key: 'components', value: 'components', icon: 'component', label: 'Components' },
] as const

export const FACET_DOORS = [
  { key: 'color', facet: 'color', icon: 'palette', label: 'Color', hint: 'Brand, surfaces, semantic' },
  { key: 'shape', facet: 'shape', icon: 'shapes', label: 'Shape', hint: 'Radius, elevation, material' },
  { key: 'assets', facet: 'assets', icon: 'sparkles', label: 'Assets', hint: 'Type and icons' },
  { key: 'components', facet: 'components', icon: 'component', label: 'Components', hint: 'L1 overrides and preview' },
] as const

const FACET_CRUMB: Record<ThemeFacet, string> = {
  overview: 'Overview',
  color: 'Color',
  shape: 'Shape',
  assets: 'Assets',
  components: 'Components',
}

export function facetCrumb(facet: ThemeFacet): string {
  return FACET_CRUMB[facet]
}

/** Build href for a theme facet (or overview root). */
export function themeFacetHref(themeId: string, facet: ThemeFacet | 'overview' = 'overview'): string {
  if (!themeId) return hrefForActivity('themes')
  const base = appRouteBase()
  const root = `${base}/themes/${encodeURIComponent(themeId)}`
  if (facet === 'overview') return root
  return `${root}/${facet}`
}

export function themesHomeHref(): string {
  return hrefForActivity('themes')
}
