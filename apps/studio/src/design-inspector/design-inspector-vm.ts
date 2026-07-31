/**
 * Design inspector VM — live specimen side panel + scale-family tools.
 */
import { mountAmbient, signal } from '@ship/sail-runtime'
import {
  addFontFamily,
  filteredIconGlyphs,
  forkTheme,
  getDesignState,
  resetInheritance,
  searchFonts,
  setComponentOverride,
  setDesignDensity,
  setIconPack,
  setIconQuery,
  setIconScaleStep,
  setInspectOpen,
  setRadiusStep,
  setScaleTogether,
  setSpecimenPreviewRoot,
  setTypeRole,
  subscribeDesign,
  uiActiveGroup,
  uiHeader,
  uiIconScaleSteps,
  uiInheritance,
  uiRadiusSteps,
  uiThemeOptions,
  uiTokenRows,
  type DensityId,
} from '@ship/studio/design'

type RadiusStepUi = {
  key: string
  valuePx: number
  raw: string
  label: string
  rowKey: string
}
type InheritUi = {
  id: string
  label: string
  isOverride: boolean
  canReset: boolean
  value: string
  key: string
}
type ThemeOptUi = { id: string; label: string; key: string }
type SiteUi = { file: string; key: string }
type OverrideUi = {
  path: string
  label: string
  display: string
  value: string
  isColor: boolean
  key: string
}

export function createDesignInspector() {
  const designThemeOpts = signal<ThemeOptUi[]>([])
  const designThemeId = signal('ship')
  const radiusSteps = signal<RadiusStepUi[]>([])
  const iconScaleSteps = signal<RadiusStepUi[]>([])
  const inheritRows = signal<InheritUi[]>([])
  const densityCompact = signal(false)
  const densityDefault = signal(true)
  const densitySpacious = signal(false)
  const scaleTogetherOn = signal(true)
  const blastLabel = signal('Not used in this project')
  const inspectOpen = signal(false)
  const inspectSites = signal<SiteUi[]>([])
  const isType = signal(false)
  const isDensity = signal(false)
  const isRadius = signal(false)
  const isIcons = signal(false)
  const isColor = signal(true)
  const showScaleFamily = signal(false)
  const typeDisplay = signal('Syne')
  const typeBody = signal('DM Sans')
  const typeMono = signal('JetBrains Mono')
  const iconPack = signal('lucide')
  const iconQuery = signal('')
  const iconGlyphs = signal<{ name: string; key: string }[]>([])
  const forkName = signal('my-brand')
  const fontSearchQ = signal('')
  const fontHits = signal<{ name: string; key: string }[]>([])
  const fontNote = signal('Search and add a font family')
  const designDensity = signal('default')
  const overrideRows = signal<OverrideUi[]>([])
  const hasOverrides = signal(false)
  const groupLabel = signal('Color')

  const sync = () => {
    const ds = getDesignState()
    const g = uiActiveGroup()
    designThemeOpts.set(
      uiThemeOptions().map((t) => ({ id: t.id, label: t.label, key: t.id })),
    )
    designThemeId.set(ds.themeId)
    radiusSteps.set(uiRadiusSteps().map((s) => ({ ...s, rowKey: s.key })))
    iconScaleSteps.set(
      uiIconScaleSteps().map((s) => ({ ...s, rowKey: `icon-${s.key}` })),
    )
    inheritRows.set(
      uiInheritance().map((n) => ({
        id: n.id,
        label: n.label,
        isOverride: n.isOverride,
        canReset: n.canReset,
        value: n.value ?? '',
        key: n.id,
      })),
    )
    densityCompact.set(ds.density === 'compact')
    densityDefault.set(ds.density === 'default')
    densitySpacious.set(ds.density === 'spacious')
    designDensity.set(ds.density)
    scaleTogetherOn.set(ds.radius.scaleTogether)
    blastLabel.set(ds.blast?.label ?? 'Not used in this project')
    inspectOpen.set(ds.inspectOpen)
    inspectSites.set(
      (ds.blast?.sites ?? []).slice(0, 40).map((s, i) => ({
        file: s.file,
        key: `${s.file}-${i}`,
      })),
    )
    isType.set(g.isType)
    isDensity.set(g.isDensity)
    isRadius.set(g.isRadius)
    isIcons.set(g.isIcon)
    isColor.set(g.isColor)
    showScaleFamily.set(g.isRadius || g.isIcon)
    groupLabel.set(g.label)
    typeDisplay.set(ds.typeRoles.display)
    typeBody.set(ds.typeRoles.body)
    typeMono.set(ds.typeRoles.mono)
    iconPack.set(ds.icons.pack)
    iconQuery.set(ds.icons.query)
    iconGlyphs.set(filteredIconGlyphs().map((name) => ({ name, key: name })))
    const ovr = uiTokenRows()
      .filter((r) => r.dirty)
      .slice(0, 40)
      .map((r) => ({
        path: r.path,
        label: r.label ?? r.path,
        display: r.display ?? r.value,
        value: r.value,
        isColor: !!r.isColor,
        key: r.path,
      }))
    overrideRows.set(ovr)
    hasOverrides.set(ovr.length > 0)
    void uiHeader()
  }

  sync()

  mountAmbient(() => {
    const unsub = subscribeDesign(() => sync())
    const bindRoot = () => {
      if (typeof document === 'undefined') return
      const el = document.querySelector(
        '[data-specimen-live]',
      ) as HTMLElement | null
      if (el) setSpecimenPreviewRoot(el)
    }
    bindRoot()
    const id = setInterval(bindRoot, 500)
    const paint = () => {
      if (typeof document === 'undefined') return
      for (const el of document.querySelectorAll<HTMLElement>('[data-swatch]')) {
        const v = el.dataset.swatch
        if (v) el.style.background = v
      }
    }
    const paintId = setInterval(paint, 200)
    paint()
    return () => {
      unsub()
      clearInterval(id)
      clearInterval(paintId)
      setSpecimenPreviewRoot(null)
    }
  })

  const onDensityClick = (e: Event) => {
    const t = (e.target as Element | null)?.closest?.('[data-density]') as HTMLElement | null
    if (!t?.dataset.density) return
    void setDesignDensity(t.dataset.density as DensityId).then(() => sync())
  }

  const onScaleTogether = (e: Event) => {
    setScaleTogether((e.target as HTMLInputElement).checked)
    sync()
  }

  const onRadiusInput = (e: Event) => {
    const t = e.target as HTMLInputElement
    const key = t.dataset.radiusStep
    if (!key) return
    const px = Number(t.value)
    if (Number.isNaN(px)) return
    void setRadiusStep(key, px).then(() => sync())
  }

  const onInspectToggle = () => {
    setInspectOpen(!getDesignState().inspectOpen)
    sync()
  }

  const onInheritReset = (e: Event) => {
    const t = (e.target as Element | null)?.closest?.('[data-inherit-id]') as HTMLElement | null
    if (!t?.dataset.inheritId) return
    void resetInheritance(t.dataset.inheritId).then(() => sync())
  }

  const onInheritOverride = (e: Event) => {
    const t = (e.target as Element | null)?.closest?.('[data-inherit-id]') as HTMLElement | null
    if (!t?.dataset.inheritId) return
    const v = t.dataset.inheritId === 'button' ? '9999px' : '12px'
    void setComponentOverride(t.dataset.inheritId, v).then(() => sync())
  }

  const onForkTheme = () => {
    const name = forkName().trim() || 'my-brand'
    void forkTheme(name).then(() => sync())
  }

  const onForkName = (e: Event) => {
    forkName.set((e.target as HTMLInputElement).value)
  }

  const onIconScaleInput = (e: Event) => {
    const t = e.target as HTMLInputElement
    const key = t.dataset.iconStep
    if (!key) return
    const px = Number(t.value)
    if (Number.isNaN(px)) return
    void setIconScaleStep(key, px).then(() => sync())
  }

  const onTypeRole = (e: Event) => {
    const t = e.target as HTMLInputElement
    const role = t.dataset.typeRole as 'display' | 'body' | 'mono' | undefined
    if (!role) return
    void setTypeRole(role, t.value).then(() => sync())
  }

  const onFontSearch = (e: Event) => {
    const q = (e.target as HTMLInputElement).value
    fontSearchQ.set(q)
    void searchFonts(q).then((families) => {
      fontHits.set(families.slice(0, 12).map((name) => ({ name, key: name })))
      sync()
    })
  }

  const onFontAdd = () => {
    const q = fontSearchQ().trim()
    if (!q) return
    void addFontFamily(q).then((r) => {
      fontNote.set(r.message)
      sync()
    })
  }

  const onFontHit = (e: Event) => {
    const name = (e.currentTarget as HTMLElement).getAttribute('data-font-hit')?.trim()
    if (!name) return
    fontSearchQ.set(name)
    void addFontFamily(name).then((r) => {
      fontNote.set(r.message)
      sync()
    })
  }

  const onIconPack = (e: Event) => {
    setIconPack((e.target as HTMLSelectElement).value)
    sync()
  }

  const onIconQuery = (e: Event) => {
    setIconQuery((e.target as HTMLInputElement).value)
    sync()
  }

  return {
    designThemeOpts,
    designThemeId,
    radiusSteps,
    iconScaleSteps,
    inheritRows,
    densityCompact,
    densityDefault,
    densitySpacious,
    scaleTogetherOn,
    blastLabel,
    inspectOpen,
    inspectSites,
    isType,
    isDensity,
    isRadius,
    isIcons,
    isColor,
    showScaleFamily,
    typeDisplay,
    typeBody,
    typeMono,
    iconPack,
    iconQuery,
    iconGlyphs,
    forkName,
    fontSearchQ,
    fontHits,
    fontNote,
    designDensity,
    overrideRows,
    hasOverrides,
    groupLabel,
    onDensityClick,
    onScaleTogether,
    onRadiusInput,
    onInspectToggle,
    onInheritReset,
    onInheritOverride,
    onForkTheme,
    onForkName,
    onIconScaleInput,
    onTypeRole,
    onFontSearch,
    onFontAdd,
    onFontHit,
    onIconPack,
    onIconQuery,
  }
}
