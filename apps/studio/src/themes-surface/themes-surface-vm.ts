/**
 * Themes surface VM — project home master–detail (Variant B).
 * Reuses @ship/studio/design store: loadThemes / openTheme / addThemeToProject.
 *
 * Paint mode is per-theme (from theme meta), not the studio shell mode.
 * Light-only themes (paper, glass, halo, brut, …) have no dark CSS tables —
 * forcing data-mode=dark makes preview fall through to :root (studio).
 *
 * Membership model = project file on disk (catalog vs project).
 * Delete + Add to project require confirm. UI never says "fork".
 * No fake enable-off switch; no session-default star theater.
 */
import { mountAmbient, signal } from '@ship/sail-runtime'
import { navigate } from '@ship/deck/router'
import {
  addThemeToProject,
  deleteTheme,
  getDesignState,
  loadThemes,
  openTheme,
  subscribeDesign,
  type ThemeFile,
} from '@ship/studio/design'
import { THEMES } from '@ship/tokens/themes'
import { appRouteBase } from '../shell/shell-page-vm'
import {
  confirmAddToProject,
  confirmDeleteTheme,
  confirmNewTheme,
  confirmRemixTheme,
} from '../theme-session/theme-confirm'
import {
  modesForPaintId,
  resolvePaintId,
  type ModeInfo,
} from '../theme-session/theme-paint'

export type ThemeRowUi = {
  id: string
  name: string
  meta: string
  blurb: string
  statusLabel: string
  statusTone: string
  selected: boolean
  /** True when theme has a project file (not catalog-only). */
  inProject: boolean
  isCatalog: boolean
  catalog: boolean
  /** Registered catalog id for theme= paint (registered tables). */
  previewId: string
  /** Mode that has a generated CSS table for this theme. */
  paintMode: 'light' | 'dark'
  /** e.g. "Light only" | "Light · Dark" */
  modesLabel: string
  menuId: string
  key: string
}

const THEME_BLURBS: Record<string, string> = {
  studio: 'Warm product chrome for day-to-day building.',
  ship: 'Cool SHIP defaults — slate surfaces and amber primary.',
  sail: 'Sail brand — the default identity for new apps.',
  neutral: 'Quiet grays. Lets content and one accent do the work.',
  paper: 'Print-inspired light surfaces and editorial type feel.',
  halo: 'Soft luminous surfaces with gentle depth.',
  console: 'Terminal-dark phosphor energy for dense tooling.',
  contrast: 'High-contrast accessibility-first pairings.',
  glass: 'Translucent panels and soft refraction cues.',
  brut: 'Brutalist edges — hard borders, strong type.',
  '8bit': 'Retro game-adjacent palette and hard pixels.',
  synthwave: 'Neon dusk — magenta, cyan, night sky.',
}

function titleCase(id: string): string {
  return (
    id
      .split(/[-_]/)
      .filter(Boolean)
      .map((p) => p.charAt(0).toUpperCase() + p.slice(1))
      .join(' ') || id
  )
}

/** Human label + blurb + paint id for a theme id (registered or project copy). */
function describeTheme(
  id: string,
  forkedFrom?: string | null,
): {
  name: string
  blurb: string
  previewId: string
  modeInfo: ModeInfo
  paintWarning?: string
} {
  const reg = THEMES.find((t) => t.id === id)
  const resolved = resolvePaintId(id, forkedFrom)
  const previewId = resolved.paintId
  const parent = THEMES.find((t) => t.id === previewId)
  if (reg) {
    return {
      name: reg.label,
      blurb: THEME_BLURBS[id] ?? `${reg.label} — full token bundle from the Sail catalog.`,
      previewId: id,
      modeInfo: modesForPaintId(id),
    }
  }
  return {
    name: titleCase(id),
    blurb: parent
      ? `Project theme based on ${parent.label}. Open to edit Color, Shape, and other facets; live draft paint uses applyScope when available.`
      : 'Project theme. Open it to edit DTCG tokens in Themes.',
    previewId,
    modeInfo: modesForPaintId(previewId),
    paintWarning: resolved.warning,
  }
}

export function createThemesSurface() {
  const projectRows = signal<ThemeRowUi[]>([])
  const catalogRows = signal<ThemeRowUi[]>([])
  const hasProjectRows = signal(false)
  const hasCatalogRows = signal(false)
  const hasAnyThemes = signal(false)
  const selectedId = signal('')
  const hasSelected = signal(false)
  const previewTitle = signal('Select a theme')
  const previewDesc = signal('')
  const previewMeta = signal('')
  const previewStatusLabel = signal('')
  const previewStatusTone = signal('muted')
  /** True when selected theme is in the project (file on disk). */
  const previewInProject = signal(false)
  const previewIsCatalog = signal(false)
  const previewThemeId = signal('studio')
  const previewPaintId = signal('studio')
  /** Paint mode for the selected theme (clamped to that theme’s tables). */
  const previewModeId = signal<'light' | 'dark'>('dark')
  const previewModesLabel = signal('')
  const canToggleMode = signal(false)
  const modeLightOn = signal(false)
  const modeDarkOn = signal(false)
  /** Ephemeral feedback — fixed toast overlay (not an in-flow status band). */
  const showToast = signal(false)
  const toastMessage = signal('')
  const toastTone = signal('muted')
  /** Bumped on each setNote so <for key> remounts toast + restarts autoDismiss. */
  type ToastSlot = { key: string; message: string; tone: string }
  const toastItems = signal<ToastSlot[]>([])
  let toastSeq = 0
  const themesError = signal('')
  const hasThemesError = signal(false)
  const loadingLabel = signal('')
  const isLoading = signal(false)
  const projectCountLabel = signal('0')
  const catalogCountLabel = signal('0')

  const menuTargetId = signal('')

  /**
   * User-chosen mode when theme supports both. Keyed by paint id.
   * Single-mode themes ignore this and always use their only table.
   */
  const modeByPaintId = new Map<string, 'light' | 'dark'>()
  /** Avoid re-toasting the same paint fallback on every sync. */
  let paintWarnForId = ''

  const resolvePaintMode = (paintId: string): 'light' | 'dark' => {
    const info = modesForPaintId(paintId)
    if (!info.dual) return info.defaultMode
    return modeByPaintId.get(paintId) ?? info.defaultMode
  }

  const mapRow = (
    t: ThemeFile,
    sel: string,
    open?: { themeId: string; forkedFrom: string | null },
  ): ThemeRowUi => {
    const inProject = !t.catalog
    // Catalog chip only; project rows have no fake "Default" / enable status.
    const statusLabel = inProject ? '' : 'Catalog'
    const statusTone = 'muted'
    // Store forkedFrom applies only to the currently open theme row.
    const rowFork =
      open?.forkedFrom && open.themeId === t.id ? open.forkedFrom : null
    const d = describeTheme(t.id, rowFork)
    const paintMode = resolvePaintMode(d.previewId)
    const meta = inProject
      ? d.modeInfo.modesLabel
      : `Catalog · ${d.modeInfo.modesLabel}`
    return {
      id: t.id,
      name: d.name,
      meta,
      blurb: d.blurb,
      statusLabel,
      statusTone,
      selected: sel === t.id,
      inProject,
      isCatalog: t.catalog,
      catalog: t.catalog,
      previewId: d.previewId,
      paintMode,
      modesLabel: d.modeInfo.modesLabel,
      menuId: `theme-menu-${t.id}`,
      key: t.id,
    }
  }

  const allRows = (): ThemeRowUi[] => [...projectRows(), ...catalogRows()]

  const sync = () => {
    const ds = getDesignState()
    const sel =
      selectedId() ||
      ds.themeId ||
      ds.projectThemes[0]?.id ||
      ds.catalogThemes[0]?.id ||
      ''
    if (sel && selectedId() !== sel) selectedId.set(sel)

    const open = { themeId: ds.themeId || '', forkedFrom: ds.forkedFrom }
    const proj = ds.projectThemes.map((t) => mapRow(t, sel, open))
    const projIds = new Set(ds.projectThemes.map((t) => t.id))
    const cat = ds.catalogThemes
      .filter((t) => !projIds.has(t.id))
      .map((t) => mapRow(t, sel, open))

    projectRows.set(proj)
    catalogRows.set(cat)
    hasProjectRows.set(proj.length > 0)
    hasCatalogRows.set(cat.length > 0)
    hasAnyThemes.set(proj.length + cat.length > 0)
    projectCountLabel.set(String(proj.length))
    catalogCountLabel.set(String(cat.length))

    const row = allRows().find((r) => r.id === sel) ?? null
    hasSelected.set(!!row)
    if (row) {
      // Re-resolve for selected preview so forkedFrom + warning apply to detail pane.
      const selectedFork =
        open.forkedFrom && row.id === open.themeId ? open.forkedFrom : null
      const resolved = resolvePaintId(row.id, selectedFork)
      const previewId = resolved.paintId
      const info = modesForPaintId(previewId)
      const mode = resolvePaintMode(previewId)
      previewTitle.set(row.name)
      previewDesc.set(row.blurb)
      previewMeta.set(row.meta)
      previewStatusLabel.set(row.statusLabel)
      previewStatusTone.set(row.statusTone)
      previewInProject.set(row.inProject)
      previewIsCatalog.set(row.isCatalog)
      previewThemeId.set(row.id)
      previewPaintId.set(previewId)
      previewModeId.set(mode)
      previewModesLabel.set(info.modesLabel)
      canToggleMode.set(info.dual)
      modeLightOn.set(mode === 'light')
      modeDarkOn.set(mode === 'dark')
      if (resolved.warning && paintWarnForId !== row.id) {
        paintWarnForId = row.id
        setNote(resolved.warning, 'warning')
      }
    } else {
      previewTitle.set('Select a theme')
      previewDesc.set('Choose a theme on the left to preview its surfaces, type, and controls.')
      previewMeta.set('')
      previewStatusLabel.set('')
      previewStatusTone.set('muted')
      previewInProject.set(false)
      previewIsCatalog.set(false)
      previewThemeId.set(ds.themeId || 'studio')
      previewPaintId.set(ds.themeId || 'studio')
      previewModeId.set('dark')
      previewModesLabel.set('')
      canToggleMode.set(false)
      modeLightOn.set(false)
      modeDarkOn.set(false)
    }

    const err = ds.error || ds.lastWriteError
    themesError.set(err ?? '')
    hasThemesError.set(!!err)
    isLoading.set(ds.loading)
    loadingLabel.set(ds.loading ? 'Loading themes…' : '')
  }

  const setNote = (msg: string, tone = 'muted') => {
    if (!msg) {
      showToast.set(false)
      toastMessage.set('')
      toastTone.set('muted')
      toastItems.set([])
      return
    }
    toastSeq += 1
    const item: ToastSlot = { key: `toast-${toastSeq}`, message: msg, tone }
    toastMessage.set(msg)
    toastTone.set(tone)
    toastItems.set([item])
    showToast.set(true)
  }

  const onToastDismiss = () => {
    showToast.set(false)
    toastMessage.set('')
    toastItems.set([])
  }

  const selectTheme = (id: string) => {
    if (!id) return
    selectedId.set(id)
    sync()
  }

  const applyPreviewMode = (m: 'light' | 'dark') => {
    const paintId = previewPaintId()
    const info = modesForPaintId(paintId)
    if (!info.modes.includes(m)) {
      setNote(`${previewTitle()} only ships a ${info.modesLabel.toLowerCase()} table.`)
      return
    }
    modeByPaintId.set(paintId, m)
    sync()
  }

  /** Switch checked = dark; unchecked = light (matches facet chrome). */
  const onPreviewModeSwitch = (e: Event) => {
    const checked = (e.target as HTMLInputElement).checked
    applyPreviewMode(checked ? 'dark' : 'light')
  }

  const onListClick = (e: Event) => {
    const el = (e.target as Element | null)?.closest?.('[theme-id]') as HTMLElement | null
    if (!el?.dataset.themeId) return
    const ctrl = (e.target as Element | null)?.closest?.(
      'switch, [role="switch"], button, menu-item, [sail-host]="menu-item"], a, input, [popover]',
    )
    if (ctrl && ctrl !== el) return
    selectTheme(el.dataset.themeId)
  }

  const onMenuOpen = (e: Event) => {
    e.stopPropagation()
    const el = (e.target as Element | null)?.closest?.('[theme-id]') as HTMLElement | null
    const id = el?.dataset.themeId
    if (id) {
      menuTargetId.set(id)
      selectTheme(id)
    }
  }

  /** Catalog → project file via confirm + addThemeToProject. */
  const addCatalogToProject = (id: string) => {
    const row = allRows().find((r) => r.id === id)
    if (!row || row.inProject) return
    const newId = id.startsWith('my-') ? id : `my-${id}`
    const srcName = describeTheme(id).name
    if (!confirmAddToProject(srcName, newId)) {
      setNote('Add to project cancelled.')
      return
    }
    setNote(`Adding ${srcName} to the project…`)
    void addThemeToProject(newId, id).then((ok) => {
      if (ok) {
        selectedId.set(newId)
        setNote(`${describeTheme(newId).name} is in the project.`, 'success')
      } else {
        setNote(
          getDesignState().lastWriteError || 'Could not add that theme to the project.',
          'destructive',
        )
      }
      sync()
    })
  }

  /** List-row Add to project (catalog only). */
  const onAddToProject = (e: Event) => {
    e.stopPropagation()
    const el = (e.target as Element | null)?.closest?.('[theme-id]') as HTMLElement | null
    const id = el?.dataset.themeId
    if (!id) return
    selectTheme(id)
    addCatalogToProject(id)
  }

  const onOpenTheme = (id?: string) => {
    const tid = id || selectedId()
    if (!tid) return
    void openTheme(tid).then(() => {
      const err = getDesignState().error
      if (err) {
        setNote(err, 'destructive')
        selectTheme(tid)
        sync()
        return
      }
      selectTheme(tid)
      sync()
      const base = appRouteBase()
      navigate(`${base}/themes/${encodeURIComponent(tid)}`)
    })
  }

  const onPreviewOpen = () => onOpenTheme()

  const onPreviewAddToProject = () => {
    const id = selectedId()
    if (!id) return
    addCatalogToProject(id)
  }

  const onNewTheme = () => {
    const src =
      selectedId() ||
      getDesignState().themeId ||
      getDesignState().catalogThemes[0]?.id ||
      'studio'
    const stamp = Date.now().toString(36).slice(-4)
    const newId = `my-theme-${stamp}`
    const srcName = describeTheme(src).name
    if (!confirmNewTheme(srcName, newId)) {
      setNote('New theme cancelled.')
      return
    }
    setNote('Creating project theme…')
    void addThemeToProject(newId, src).then((ok) => {
      if (ok) {
        selectedId.set(newId)
        setNote(`Created ${describeTheme(newId).name} from ${srcName}.`, 'success')
      } else {
        setNote(getDesignState().lastWriteError || 'Could not create theme.', 'destructive')
      }
      sync()
    })
  }

  const onRowMenu = (e: Event) => {
    const ce = e as CustomEvent<{ value?: string }>
    const cmd = ce.detail?.value || ''
    const id = menuTargetId() || selectedId()
    if (!cmd || !id) return
    selectTheme(id)
    if (cmd === 'open') onOpenTheme(id)
    else if (cmd === 'enable' || cmd === 'add') {
      // Menu primary for catalog: Add to project
      addCatalogToProject(id)
    } else if (cmd === 'remix' || cmd === 'duplicate') {
      const stamp = Date.now().toString(36).slice(-4)
      const newId = `my-${id}-${stamp}`.replace(/[^a-z0-9-]/g, '-').replace(/-+/g, '-')
      const srcName = describeTheme(id).name
      if (!confirmRemixTheme(srcName, newId)) {
        setNote('Remix cancelled.')
        return
      }
      void addThemeToProject(newId, id).then((ok) => {
        if (ok) {
          setNote(`Remixed as ${describeTheme(newId).name}`, 'success')
          selectedId.set(newId)
        } else {
          setNote(getDesignState().lastWriteError || 'Remix failed', 'destructive')
        }
        sync()
      })
    } else if (cmd === 'delete') {
      const row = allRows().find((r) => r.id === id)
      if (row?.isCatalog) {
        setNote('Catalog themes cannot be deleted.', 'warning')
        return
      }
      if (!confirmDeleteTheme(describeTheme(id).name)) {
        setNote('Delete cancelled.')
        return
      }
      void deleteTheme(id).then((ok) => {
        if (ok) {
          if (selectedId() === id) selectedId.set('')
          setNote('Theme deleted.', 'success')
        } else {
          setNote(getDesignState().lastWriteError || 'Could not delete theme.', 'destructive')
        }
        sync()
      })
    }
  }

  const onRetryThemes = () => {
    void loadThemes().then(() => {
      setNote('Theme list reloaded.')
      sync()
    })
  }

  sync()

  mountAmbient(() => {
    const unsub = subscribeDesign(() => sync())
    const ds = getDesignState()
    if (ds.projectThemes.length === 0 && ds.catalogThemes.length === 0 && !ds.loading) {
      void loadThemes().then(() => sync())
    }
    return () => unsub()
  })

  return {
    projectRows,
    catalogRows,
    hasProjectRows,
    hasCatalogRows,
    hasAnyThemes,
    selectedId,
    hasSelected,
    previewTitle,
    previewDesc,
    previewMeta,
    previewStatusLabel,
    previewStatusTone,
    previewInProject,
    /** @deprecated alias — project membership, not an enable flag */
    previewEnabled: previewInProject,
    previewIsCatalog,
    previewThemeId,
    previewPaintId,
    previewModeId,
    previewModesLabel,
    canToggleMode,
    modeLightOn,
    modeDarkOn,
    showToast,
    toastMessage,
    toastTone,
    toastItems,
    themesError,
    hasThemesError,
    loadingLabel,
    isLoading,
    projectCountLabel,
    catalogCountLabel,
    onListClick,
    onMenuOpen,
    onAddToProject,
    onPreviewOpen,
    onPreviewAddToProject,
    /** @deprecated alias for template migration */
    onPreviewEnable: onPreviewAddToProject,
    onPreviewModeSwitch,
    onNewTheme,
    onRowMenu,
    onRetryThemes,
    onToastDismiss,
  }
}
