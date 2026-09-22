/**
 * Shape recipes + elev shadow fallbacks — data only.
 */
export const ELEV_STEPS = [
  { key: 'none', label: 'none', hint: 'flat' },
  { key: 'xs', label: 'xs', hint: 'hair' },
  { key: 'sm', label: 'sm', hint: 'rest' },
  { key: 'md', label: 'md', hint: 'card' },
  { key: 'lg', label: 'lg', hint: 'float' },
  { key: 'xl', label: 'xl', hint: 'modal' },
] as const

export type ElevKey = (typeof ELEV_STEPS)[number]['key']
export const RADIUS_KEYS = ['sm', 'md', 'lg', 'xl'] as const
export type RadiusKey = (typeof RADIUS_KEYS)[number]
export const RECIPE_IDS = ['soft', 'sharp', 'glass', 'flat', 'elevated'] as const
export type RecipeId = (typeof RECIPE_IDS)[number] | 'custom'

export const RECIPES: Record<
  Exclude<RecipeId, 'custom'>,
  { master: number; elev: ElevKey; borderW: number; glass: boolean; surfaceOp: number; surfaceBlur: number }
> = {
  soft: { master: 10, elev: 'md', borderW: 1, glass: false, surfaceOp: 1, surfaceBlur: 0 },
  sharp: { master: 2, elev: 'sm', borderW: 1.5, glass: false, surfaceOp: 1, surfaceBlur: 0 },
  glass: { master: 12, elev: 'lg', borderW: 1, glass: true, surfaceOp: 0.72, surfaceBlur: 14 },
  flat: { master: 6, elev: 'none', borderW: 1, glass: false, surfaceOp: 1, surfaceBlur: 0 },
  elevated: { master: 10, elev: 'xl', borderW: 0.5, glass: false, surfaceOp: 1, surfaceBlur: 0 },
}

export const SHADOW_FALLBACKS: Record<Exclude<ElevKey, 'none'>, string> = {
  xs: '0 1px 2px 0 oklch(0 0 0 / 5%)',
  sm: '0 1px 3px 0 oklch(0 0 0 / 7%), 0 1px 2px -1px oklch(0 0 0 / 5%)',
  md: '0 4px 12px -2px oklch(0 0 0 / 9%), 0 2px 4px -2px oklch(0 0 0 / 6%)',
  lg: '0 12px 32px -6px oklch(0 0 0 / 13%), 0 4px 12px -4px oklch(0 0 0 / 8%)',
  xl: '0 24px 64px -12px oklch(0 0 0 / 18%), 0 8px 24px -8px oklch(0 0 0 / 10%)',
}

export function elevFromCardRest(raw: string | null): ElevKey {
  if (!raw || raw === 'none') return 'none'
  const norm = raw.replace(/\s+/g, ' ').trim()
  for (const key of ['xl', 'lg', 'md', 'sm', 'xs'] as const) {
    const fb = SHADOW_FALLBACKS[key].replace(/\s+/g, ' ').trim()
    if (norm === fb || norm.includes(fb.slice(0, 18))) return key
  }
  if (/\b0\s+24px\b/.test(norm)) return 'xl'
  if (/\b0\s+12px\b/.test(norm)) return 'lg'
  if (/\b0\s+4px\b/.test(norm)) return 'md'
  if (/\b0\s+1px\s+3px\b/.test(norm)) return 'sm'
  if (/\b0\s+1px\s+2px\b/.test(norm)) return 'xs'
  return 'md'
}
