/**
 * Assets domain — deep type roles + icon pack / size / curated glyphs.
 * Font discoverer: full Google Fonts catalog via SailTheme/FontSearch (bundled),
 * live CSS preview, assign to display/body/mono, optional FontAdd vendor.
 */
import { signal } from '@ship/sail-runtime'
import {
  addFontFamily,
  getDesignState,
  searchFonts,
  setIconPack,
  setIconScaleStep,
  setTypeRole,
  uiIconScaleSteps,
} from '@ship/studio/design'
import type { ThemeSession } from '../theme-session/theme-session'

const CURATED_GLYPHS = [
  'home',
  'search',
  'settings',
  'user',
  'bell',
  'mail',
  'folder',
  'star',
  'plus',
  'check',
  'x',
  'chevron-right',
  'more-horizontal',
  'trash',
  'edit',
  'download',
] as const

export type FontOpt = { value: string; label: string; key: string }

export type FontHitUi = {
  key: string
  family: string
  category: string
  mono: boolean
  sample: string
}

/** Real typeface name only — never CSS cascade, DTCG braces, or token cssNames. */
function isFaceName(v: string): boolean {
  const t = v.trim()
  if (!t) return false
  if (t.startsWith('{') || t.includes('var(') || t.startsWith('--')) return false
  if (t.startsWith('font-') || t.endsWith('-font') || t.endsWith('-app')) return false
  return true
}

const FONT_CATEGORIES = [
  { key: 'all', id: 'all', label: 'All' },
  { key: 'sans', id: 'sans', label: 'Sans' },
  { key: 'serif', id: 'serif', label: 'Serif' },
  { key: 'display', id: 'display', label: 'Display' },
  { key: 'mono', id: 'mono', label: 'Mono' },
  { key: 'handwriting', id: 'handwriting', label: 'Script' },
] as const

const PREVIEW_SAMPLE = 'Ag The quick brown fox jumps 0123'

function fontOpts(list: string[], current: string): FontOpt[] {
  const seen = new Set<string>()
  const vals: string[] = []
  for (const v of [current, ...list]) {
    if (!v || !isFaceName(v) || seen.has(v)) continue
    seen.add(v)
    vals.push(v)
  }
  return vals.map((v) => ({ value: v, label: v, key: v }))
}

/**
 * Live font preview inject — ambient document/CSSOM banned (DOMDEC).
 * No-op until a platform font-preview primitive lands; samples use theme tokens.
 */
function ensurePreviewFaces(_families: string[]) {
  /* gap: no document.createElement / querySelector in apps */
}

export function createAssetsEdit(session: ThemeSession) {
  const typeDisplay = signal('Inter')
  const typeBody = signal('Inter')
  const typeMono = signal('JetBrains Mono')
  /** Families available in role selects (current + discovered + installed). */
  const knownFamilies = signal<string[]>([
    'Inter',
    'Fraunces',
    'DM Sans',
    'Source Serif 4',
    'Syne',
    'IBM Plex Sans',
    'JetBrains Mono',
    'Geist Mono',
    'IBM Plex Mono',
  ])
  const displayFontOpts = signal<FontOpt[]>(fontOpts(knownFamilies(), 'Inter'))
  const bodyFontOpts = signal<FontOpt[]>(fontOpts(knownFamilies(), 'Inter'))
  const monoFontOpts = signal<FontOpt[]>(fontOpts(knownFamilies(), 'JetBrains Mono'))
  const iconPack = signal('lucide')
  const iconQuery = signal('')
  const iconPreviewSize = signal<'sm' | 'md' | 'lg' | 'xl'>('md')
  const iconSizeSmOn = signal(false)
  const iconSizeMdOn = signal(true)
  const iconSizeLgOn = signal(false)
  const iconSizeXlOn = signal(false)
  type GlyphUi = { name: string; key: string }
  const iconGlyphs = signal<GlyphUi[]>([])
  const hasIconGlyphs = signal(false)
  const fontSearchQ = signal('')
  const fontCategory = signal('all')
  const fontSearchHits = signal<FontHitUi[]>([])
  const hasFontHits = signal(false)
  const fontSearching = signal(false)
  const fontCatalogMeta = signal('Google Fonts catalog')
  const fontBusyFamily = signal('')
  const iconScaleReadout = signal('sm · md · lg · xl')
  const iconScaleMasterPx = signal(16)

  const rememberFamily = (family: string) => {
    if (!isFaceName(family)) return
    const next = fontOpts([...knownFamilies(), family], family).map((o) => o.value)
    knownFamilies.set(next)
  }

  const refreshRoleOpts = () => {
    const list = knownFamilies()
    displayFontOpts.set(fontOpts(list, typeDisplay()))
    bodyFontOpts.set(fontOpts(list, typeBody()))
    monoFontOpts.set(fontOpts(list, typeMono()))
  }

  const sync = () => {
    const ds = getDesignState()
    // typeRoles must already be face names (extractTypeRoles rejects CSS var cascade).
    const display = isFaceName(ds.typeRoles.display) ? ds.typeRoles.display : 'Inter'
    const body = isFaceName(ds.typeRoles.body) ? ds.typeRoles.body : 'Inter'
    const mono = isFaceName(ds.typeRoles.mono) ? ds.typeRoles.mono : 'JetBrains Mono'
    typeDisplay.set(display)
    typeBody.set(body)
    typeMono.set(mono)
    rememberFamily(display)
    rememberFamily(body)
    rememberFamily(mono)
    refreshRoleOpts()
    iconPack.set(ds.icons.pack || 'lucide')

    const steps = uiIconScaleSteps()
    if (steps.length) {
      iconScaleReadout.set(
        steps.map((s) => `${s.key} ${Math.round(s.valuePx * 10) / 10}`).join(' · '),
      )
      const md = steps.find((s) => s.key === 'md')
      if (md) iconScaleMasterPx.set(md.valuePx)
    }

    filterGlyphs(iconQuery())
  }

  const filterGlyphs = (q: string) => {
    const query = q.trim().toLowerCase()
    const all = CURATED_GLYPHS.map((name) => ({ name, key: name }))
    const filtered = query ? all.filter((g) => g.name.includes(query)) : all
    iconGlyphs.set(filtered)
    hasIconGlyphs.set(filtered.length > 0)
  }

  session.onSync(sync)
  sync()

  const onTypeSelect = (e: Event) => {
    if (!session.guardEdit()) return
    const t = e.target as HTMLSelectElement
    const role = t.getAttribute('type-token') as 'display' | 'body' | 'mono' | null
    if (!role) return
    rememberFamily(t.value)
    void setTypeRole(role, t.value).then(() => session.sync())
  }

  const onIconPack = (e: Event) => {
    if (!session.guardEdit()) return
    const v = (e.target as HTMLSelectElement).value
    setIconPack(v)
    if (v !== 'lucide') {
      session.setNote(
        `Icon pack “${v}” is session-only until pack persistence (A2).`,
        'warning',
      )
    }
    session.sync()
  }

  const onIconQuery = (e: Event) => {
    const v = (e.target as HTMLInputElement).value
    iconQuery.set(v)
    filterGlyphs(v)
  }

  /** Closed-over size — no dataset bus. */
  const onIconSize = (size: 'sm' | 'md' | 'lg' | 'xl') => {
    if (!size) return
    iconPreviewSize.set(size)
    iconSizeSmOn.set(size === 'sm')
    iconSizeMdOn.set(size === 'md')
    iconSizeLgOn.set(size === 'lg')
    iconSizeXlOn.set(size === 'xl')
  }

  const iconScaleSteps = () =>
    uiIconScaleSteps().map((s) => ({
      key: s.key,
      id: s.key,
      label: s.key,
      pxLabel: `${Math.round(s.valuePx * 10) / 10}`,
      valuePx: s.valuePx,
    }))

  const onIconScaleMaster = (e: Event) => {
    if (!session.guardEdit()) return
    const n = Number((e.target as HTMLInputElement).value)
    if (!Number.isFinite(n)) return
    iconScaleMasterPx.set(n)
    void setIconScaleStep('md', n).then(() => session.sync())
  }

  const onIconScaleStep = (key: string) => {
    if (!session.guardEdit()) return
    if (!key) return
    const steps = uiIconScaleSteps()
    const cur = steps.find((s) => s.key === key)
    if (!cur) return
    void setIconScaleStep(key, cur.valuePx).then(() => {
      if (key === 'md') iconScaleMasterPx.set(cur.valuePx)
      session.sync()
    })
  }

  let fontSearchGen = 0
  const runFontSearch = (q: string, category = fontCategory()) => {
    const query = q.trim()
    const gen = ++fontSearchGen
    fontSearching.set(true)
    const catArg = category && category !== 'all' ? category : undefined
    void searchFonts(query, { category: catArg, limit: 36 }).then((result) => {
      if (gen !== fontSearchGen) return
      fontSearching.set(false)
      // Preview via `face=` + Sail style tokens only — never authored CSS font-family strings.
      const hits: FontHitUi[] = (result.hits || [])
        .filter((h) => isFaceName(h.family))
        .map((h) => ({
          key: h.family,
          family: h.family,
          category: h.category || 'Other',
          mono: Boolean(h.mono),
          sample: PREVIEW_SAMPLE,
        }))
      fontSearchHits.set(hits)
      hasFontHits.set(hits.length > 0)
      const n = result.catalogSize || result.count
      fontCatalogMeta.set(
        n > 0
          ? `${result.count} shown · ${result.catalogSize || '—'} in Google Fonts catalog`
          : 'Font catalog unavailable',
      )
      ensurePreviewFaces(hits.map((h) => h.family))
      if (query.length >= 2 && hits.length === 0) {
        session.setNote(`No fonts matched “${query}”.`, 'muted')
      }
    })
  }

  // Seed popular fonts on first open (empty query = popularity ranking).
  runFontSearch('', 'all')

  const onFontSearchInput = (e: Event) => {
    const v = (e.target as HTMLInputElement).value
    fontSearchQ.set(v)
    // Typing a query searches the full catalog (category chips stay for browse).
    // If the user is filtering mono and types "Poppins", clear the chip so results appear.
    if (v.trim().length >= 2 && fontCategory() !== 'all') {
      fontCategory.set('all')
      runFontSearch(v, 'all')
      return
    }
    runFontSearch(v)
  }

  const onFontSearch = () => {
    runFontSearch(fontSearchQ())
  }

  const onFontCategory = (cat: string) => {
    if (!cat) return
    fontCategory.set(cat)
    runFontSearch(fontSearchQ(), cat)
  }

  const fontCategoryOpts = () =>
    FONT_CATEGORIES.map((c) => ({
      ...c,
      on: fontCategory() === c.id,
    }))

  const applyRole = async (family: string, role: 'display' | 'body' | 'mono') => {
    if (!session.guardEdit()) return
    if (!isFaceName(family)) {
      session.setNote('That is not a typeface name (CSS vars / token aliases are not fonts).', 'destructive')
      return
    }
    rememberFamily(family)
    ensurePreviewFaces([family])
    await setTypeRole(role, family)
    session.setNote(`${role}: ${family}`, 'success')
    session.sync()
  }

  const onFontUseRole = (e: Event) => {
    const el = (e.target as Element | null)?.closest?.('[font-family][font-token]') as
      | HTMLElement
      | null
    const family = el?.getAttribute('font-family') ?? undefined
    const role = el?.getAttribute('font-token') as 'display' | 'body' | 'mono' | null
    if (!family || !role) return
    void applyRole(family, role)
  }

  const onFontInstall = (e: Event) => {
    if (!session.guardEdit()) return
    const el = (e.target as Element | null)?.closest?.('[font-family]') as HTMLElement | null
    const family = el?.getAttribute('font-family') ?? undefined
    const role =
      (el?.getAttribute('font-token') as 'display' | 'body' | 'mono' | null) || 'body'
    if (!family) return
    fontBusyFamily.set(family)
    void addFontFamily(family, role).then((r) => {
      fontBusyFamily.set('')
      rememberFamily(family)
      ensurePreviewFaces([family])
      session.setNote(r.message, r.ok ? 'success' : 'warning')
      session.sync()
    })
  }

  const iconSizeOpts = () =>
    (['sm', 'md', 'lg', 'xl'] as const).map((id) => ({
      key: id,
      id,
      label: id,
      on:
        id === 'sm'
          ? iconSizeSmOn()
          : id === 'md'
            ? iconSizeMdOn()
            : id === 'lg'
              ? iconSizeLgOn()
              : iconSizeXlOn(),
    }))

  return {
    typeDisplay,
    typeBody,
    typeMono,
    displayFontOpts,
    bodyFontOpts,
    monoFontOpts,
    iconPack,
    iconQuery,
    iconPreviewSize,
    iconGlyphs,
    hasIconGlyphs,
    fontSearchQ,
    fontSearchHits,
    hasFontHits,
    fontSearching,
    fontCatalogMeta,
    fontBusyFamily,
    fontCategoryOpts,
    iconScaleReadout,
    iconScaleMasterPx,
    iconScaleSteps,
    iconSizeOpts,
    onTypeSelect,
    onIconPack,
    onIconQuery,
    onIconSize,
    onIconScaleStep,
    onIconScaleMaster,
    onFontSearchInput,
    onFontSearch,
    onFontCategory,
    onFontUseRole,
    onFontInstall,
  }
}
