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
  { key: 'overview', value: 'overview', icon: 'home', label: 'Overview' },
  { key: 'color', value: 'color', icon: 'image', label: 'Color' },
  { key: 'shape', value: 'shape', icon: 'circle', label: 'Shape' },
  { key: 'assets', value: 'assets', icon: 'sparkles', label: 'Assets' },
  { key: 'components', value: 'components', icon: 'star', label: 'Components' },
] as const

export const FACET_DOORS = [
  { key: 'color', facet: 'color', icon: 'image', label: 'Color', hint: 'Brand, surfaces, semantic' },
  { key: 'shape', facet: 'shape', icon: 'circle', label: 'Shape', hint: 'Radius, elevation, material' },
  { key: 'assets', facet: 'assets', icon: 'sparkles', label: 'Assets', hint: 'Type and icons' },
  { key: 'components', facet: 'components', icon: 'star', label: 'Components', hint: 'Component overrides and preview' },
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
