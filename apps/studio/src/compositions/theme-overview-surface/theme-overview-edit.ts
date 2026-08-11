/**
 * Overview domain — identity editors (heroes, type, icons, density).
 * Session owns open/mode/save/path; this module owns facet writes + specimen density.
 */
import { signal } from '@ship/sail-runtime'
import {
  densityLabel,
  getDesignState,
  getPathValue,
  setBrandColor,
  setDesignDensity,
  setIconPack,
  setTokenValue,
  setTypeRole,
  tryPickerHex,
  type DensityId,
} from '@ship/studio/design'
import type { ThemeSession } from '../theme-session/theme-session'
import { FACET_DOORS } from '../theme-session/theme-nav'

const SECONDARY_PATH = 'color.secondary.default'
const ACCENT_PATH = 'color.brandAccent.default'

const DISPLAY_FONTS = [
  'Fraunces',
  'Inter',
  'Cal Sans',
  'Satoshi',
  'Source Serif 4',
  'Syne',
]
const BODY_FONTS = ['DM Sans', 'Inter', 'IBM Plex Sans', 'Geist', 'Source Serif 4']
const MONO_FONTS = ['JetBrains Mono', 'Geist Mono', 'IBM Plex Mono']

const OVERVIEW_GLYPHS = [
  'home',
  'search',
  'settings',
  'user',
  'bell',
  'mail',
  'folder',
  'star',
] as const

const DENSITY_OPTS = [
  { id: 'compact' as const, label: 'Compact' },
  { id: 'default' as const, label: 'Comfortable' },
  { id: 'spacious' as const, label: 'Roomy' },
]

export type FontOpt = { value: string; label: string; key: string }

function fontOpts(list: string[], current: string): FontOpt[] {
  const vals = list.includes(current) ? list : [current, ...list]
  return vals.filter(Boolean).map((v) => ({ value: v, label: v, key: v }))
}

function colorHex(
  doc: Record<string, unknown> | null,
  path: string,
  mode: 'light' | 'dark',
  fallback: string,
): string {
  const raw = getPathValue(doc, path, mode)
  if (!raw) return fallback
  return tryPickerHex(raw) ?? ''
}

export function createOverviewEdit(session: ThemeSession) {
  const primaryHex = signal('#3b82f6')
  const secondaryHex = signal('#64748b')
  const accentHex = signal('#a855f7')
  const typeDisplay = signal('Inter')
  const typeBody = signal('Inter')
  const typeMono = signal('JetBrains Mono')
  const displayFontOpts = signal<FontOpt[]>(fontOpts(DISPLAY_FONTS, 'Inter'))
  const bodyFontOpts = signal<FontOpt[]>(fontOpts(BODY_FONTS, 'Inter'))
  const monoFontOpts = signal<FontOpt[]>(fontOpts(MONO_FONTS, 'JetBrains Mono'))
  const iconPack = signal('lucide')
  type GlyphUi = { name: string; key: string }
  const iconGlyphs = signal<GlyphUi[]>([])
  const hasIconGlyphs = signal(false)
  const densityCompact = signal(false)
  const densityDefault = signal(true)
  const densitySpacious = signal(false)
  const densityLabelSig = signal('Comfortable')
  const specimenDensity = signal('')
  // Concrete lengths only — never product var(--token) strings as state (purity).
  // Scale aligns with default 4px space ladder: 1→4, 2→8, 3→12, 5→20, 7→28.
  const densityPad = signal('20px')
  const specimenGap = signal('12px')
  const specimenStackGap = signal('8px')
  const controlSize = signal<'xs' | 'sm' | 'md'>('sm')
  const iconSize = signal<'xs' | 'sm' | 'md'>('sm')
  const badgeSize = signal<'sm' | 'md'>('sm')
  const cardPadding = signal<'none' | 'sm' | 'md'>('sm')
  const clusterGap = signal<'1' | '2' | '3'>('2')

  const sync = () => {
    const ds = getDesignState()
    const doc = ds.themeDoc
    const mode = session.writeMode()
    primaryHex.set(colorHex(doc, 'color.primary.default', mode, '#3b82f6'))
    secondaryHex.set(colorHex(doc, SECONDARY_PATH, mode, '#64748b'))
    accentHex.set(colorHex(doc, ACCENT_PATH, mode, '#a855f7'))

    const display = ds.typeRoles.display || 'Inter'
    const body = ds.typeRoles.body || 'Inter'
    const mono = ds.typeRoles.mono || 'JetBrains Mono'
    typeDisplay.set(display)
    typeBody.set(body)
    typeMono.set(mono)
    displayFontOpts.set(fontOpts(DISPLAY_FONTS, display))
    bodyFontOpts.set(fontOpts(BODY_FONTS, body))
    monoFontOpts.set(fontOpts(MONO_FONTS, mono))

    iconPack.set(ds.icons.pack || 'lucide')
    const glyphs = OVERVIEW_GLYPHS.map((name) => ({ name, key: name }))
    iconGlyphs.set(glyphs)
    hasIconGlyphs.set(glyphs.length > 0)

    densityCompact.set(ds.density === 'compact')
    densityDefault.set(ds.density === 'default')
    densitySpacious.set(ds.density === 'spacious')
    densityLabelSig.set(densityLabel(ds.density))
    specimenDensity.set(ds.density === 'default' ? '' : ds.density)
    if (ds.density === 'compact') {
      densityPad.set('12px')
      specimenGap.set('8px')
      specimenStackGap.set('4px')
      controlSize.set('xs')
      iconSize.set('xs')
      badgeSize.set('sm')
      cardPadding.set('sm')
      clusterGap.set('1')
    } else if (ds.density === 'spacious') {
      densityPad.set('28px')
      specimenGap.set('20px')
      specimenStackGap.set('12px')
      controlSize.set('md')
      iconSize.set('md')
      badgeSize.set('md')
      cardPadding.set('md')
      clusterGap.set('3')
    } else {
      densityPad.set('20px')
      specimenGap.set('12px')
      specimenStackGap.set('8px')
      controlSize.set('sm')
      iconSize.set('sm')
      badgeSize.set('sm')
      cardPadding.set('sm')
      clusterGap.set('2')
    }
  }

  session.onSync(sync)
  sync()

  const onHeroColor = (detail: { value?: string; token?: string }) => {
    if (!session.guardEdit()) return
    const hex = String(detail?.value ?? '').trim()
    if (!hex) return
    const role = detail?.token
    if (role === 'primary') {
      primaryHex.set(hex)
      void setBrandColor(hex).then(() => session.sync())
    } else if (role === 'secondary') {
      secondaryHex.set(hex)
      void setTokenValue(SECONDARY_PATH, hex).then(() => session.sync())
    } else if (role === 'accent') {
      accentHex.set(hex)
      void setTokenValue(ACCENT_PATH, hex).then(() => session.sync())
    }
  }

  const colorHeroes = () => [
    { key: 'primary', role: 'primary', label: 'Primary', hex: primaryHex() },
    { key: 'secondary', role: 'secondary', label: 'Secondary', hex: secondaryHex() },
    { key: 'accent', role: 'accent', label: 'Accent', hex: accentHex() },
  ]

  const densityOptions = () =>
    DENSITY_OPTS.map((d) => ({
      key: d.id,
      id: d.id,
      label: d.label,
      on:
        d.id === 'compact'
          ? densityCompact()
          : d.id === 'spacious'
            ? densitySpacious()
            : densityDefault(),
    }))

  const onTypeSelect = (e: Event) => {
    if (!session.guardEdit()) return
    const t = e.target as HTMLSelectElement
    const role = t.getAttribute('type-token') as 'display' | 'body' | 'mono' | null
    if (!role) return
    void setTypeRole(role, t.value).then(() => session.sync())
  }

  const onIconPack = (e: Event) => {
    if (!session.guardEdit()) return
    const v = (e.target as HTMLSelectElement).value
    setIconPack(v)
    // A2: pack is memory-only until DTCG persistence exists (same honesty as Assets).
    if (v !== 'lucide') {
      session.setNote(
        `Icon pack “${v}” is session-only until pack persistence (A2).`,
        'warning',
      )
    }
    iconGlyphs.set(OVERVIEW_GLYPHS.map((name) => ({ name, key: name })))
    hasIconGlyphs.set(true)
    session.sync()
  }

  /** Closed-over density id — no dataset bus. */
  const onDensityClick = (d: DensityId) => {
    if (!session.guardEdit()) return
    if (!d || !['compact', 'default', 'spacious'].includes(d)) return
    void setDesignDensity(d).then(() => session.sync())
  }

  return {
    primaryHex,
    secondaryHex,
    accentHex,
    colorHeroes,
    densityOptions,
    facetDoors: () => FACET_DOORS.map((p) => ({ ...p })),
    typeDisplay,
    typeBody,
    typeMono,
    displayFontOpts,
    bodyFontOpts,
    monoFontOpts,
    iconPack,
    iconGlyphs,
    hasIconGlyphs,
    densityPad,
    specimenGap,
    specimenStackGap,
    controlSize,
    iconSize,
    badgeSize,
    cardPadding,
    clusterGap,
    specimenDensity,
    densityLabel: densityLabelSig,
    onHeroColor,
    onTypeSelect,
    onIconPack,
    onDensityClick,
  }
}
