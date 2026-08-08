/**
 * Shared theme facet session — open/mode/save/path/paint/toast + dirty leave guard.
 * Only place facets call openTheme, setDesignMode, requestSaveTheme, addThemeToProject.
 * Facet overflow menu removed — delete/remix on Themes home.
 */
import { mountAmbient, signal } from '@ship/sail-runtime'
import { navigate } from '@ship/deck/router'
import {
  discardThemeEdits,
  getDesignState,
  loadThemes,
  openTheme,
  refreshBlast,
  redoThemeEdit,
  saveThemeForLeave,
  setDesignMode,
  subscribeDesign,
  undoThemeEdit,
} from '@ship/studio/design'
import {
  bindPreviewHost,
  scheduleDraftPaint,
  setDraftPaintIdResolver,
  teardownDraftPaint,
  unbindPreviewHost,
} from './theme-draft-paint'
import { createLeaveGuard } from './theme-leave-guard'
import { setThemeLeaveNavigate } from './theme-leave-bridge'
import {
  FACET_DOORS,
  PATH_NAV,
  facetCrumb,
  themeFacetHref,
  themesHomeHref,
  type ThemeFacet,
} from './theme-nav'
import {
  modesForPaintId,
  paintIdFor,
  resolvePaintId,
  themeDisplayName,
  themeIdFromLocation,
  type ModeInfo,
} from './theme-paint'
import { makeSessionShellActions } from './theme-session-actions'

export type { ThemeFacet, ModeInfo }
export { modesForPaintId, paintIdFor, resolvePaintId, themeDisplayName, themeIdFromLocation }
export {
  bindPreviewHost,
  unbindPreviewHost,
  scheduleDraftPaint,
  setDraftPaintApplier,
  teardownDraftPaint,
} from './theme-draft-paint'
export { setThemeConfirm } from './theme-confirm'

export type ToastSlot = { key: string; message: string; tone: string }

export type ThemeSessionOpts = {
  facet: ThemeFacet
  /** Guard toast when catalog theme is edited. */
  editGuardMessage?: string
  /** Save success toast. */
  saveSuccessMessage?: string
}

export type ThemeSession = ReturnType<typeof createThemeSession>

// Wire paint-id resolver once (registered tables interim path).
setDraftPaintIdResolver((id, forkedFrom) => resolvePaintId(id, forkedFrom).paintId)

export function createThemeSession(opts: ThemeSessionOpts) {
  const facet = opts.facet
  const editGuardMessage =
    opts.editGuardMessage ?? 'Add this theme to the project to edit.'
  const saveSuccessMessage = opts.saveSuccessMessage ?? 'Theme saved.'

  const themeId = signal('')
  const themeName = signal('Theme')
  const nameEdit = signal('')
  const isCatalog = signal(false)
  const canEdit = signal(false)
  /** Empty on project themes (editing is default); "Catalog" when view-only. */
  const stateChipLabel = signal('Catalog')
  const stateChipTone = signal('muted')
  const showStateChip = signal(true)
  const modesLabel = signal('Light · Dark')
  const canToggleMode = signal(false)
  const showAddMode = signal(false)
  const addModeLabel = signal('Add dark table')
  const modeLightOn = signal(true)
  const modeDarkOn = signal(false)
  const paintId = signal('studio')
  const paintMode = signal<'light' | 'dark'>('light')
  const activeModeLabel = signal('Light')
  const isDirty = signal(false)
  const canUndo = signal(false)
  const canRedo = signal(false)
  /** Unsaved path diffs for popover (from design store). */
  const diffRows = signal<
    { key: string; path: string; from: string; to: string }[]
  >([])
  const hasDiffRows = signal(false)
  const pathCrumb = signal(facetCrumb(facet))
  const loading = signal(false)
  const hasError = signal(false)
  const errorText = signal('')
  const catalogHint = signal(false)
  const modesMeta = signal('Light · Dark')
  /** Token usage label for save-review toast only — not permanent chrome. */
  const usedByLabel = signal('Not used in this project yet')
  const saveReviewOpen = signal(false)
  const saveButtonLabel = signal('Save')
  const usedBySites = signal<{ file: string; key: string }[]>([])

  /** Dirty leave dialog — Save / Discard / Cancel. */
  const leaveOpen = signal(false)
  const leaveReason = signal('')

  const toastItems = signal<ToastSlot[]>([])
  let toastSeq = 0

  /** User-chosen paint/edit mode when dual; keyed by paint id. */
  const modeByPaintId = new Map<string, 'light' | 'dark'>()
  let openedForId = ''
  /** Monotonic open generation — stale openTheme completions no-op. */
  let openGen = 0
  let renameNoteShown = false
  let paintWarnForId = ''
  /** Facet token usage refreshed once per open (color/shape). */
  let usedByRefreshedForId = ''
  const domainSyncs = new Set<() => void>()

  const resolvePaintMode = (pid: string): 'light' | 'dark' => {
    const info = modesForPaintId(pid)
    if (!info.dual) return info.defaultMode
    return modeByPaintId.get(pid) ?? info.defaultMode
  }

  const setNote = (msg: string, tone = 'muted') => {
    if (!msg) {
      toastItems.set([])
      return
    }
    toastSeq += 1
    toastItems.set([{ key: `toast-${toastSeq}`, message: msg, tone }])
  }

  const onToastDismiss = () => {
    toastItems.set([])
  }

  const notifyDomain = () => {
    for (const fn of domainSyncs) fn()
  }

  /** Domains register so session store ticks also refresh facet state. */
  const onSync = (fn: () => void) => {
    domainSyncs.add(fn)
    return () => {
      domainSyncs.delete(fn)
    }
  }

  const doOpenTheme = (id: string) => {
    if (!id) return
    openedForId = id
    const gen = ++openGen
    const run = () =>
      openTheme(id).then(() => {
        if (gen !== openGen || openedForId !== id) return
        const routeId = themeIdFromLocation()
        if (routeId && routeId !== id) return
        const err = getDesignState().error
        if (err) setNote(err, 'destructive')
        const ds = getDesignState()
        const resolved = resolvePaintId(id, ds.forkedFrom)
        if (resolved.warning && paintWarnForId !== id) {
          paintWarnForId = id
          setNote(resolved.warning, 'warning')
        }
        const mode = resolvePaintMode(resolved.paintId)
        if (getDesignState().modeId !== mode) setDesignMode(mode)
        nameEdit.set(themeDisplayName(getDesignState().themeId || id))
        const usageToken = facetUsageToken()
        if (usageToken && usedByRefreshedForId !== id) {
          usedByRefreshedForId = id
          refreshUsedBy(usageToken)
        }
        scheduleDraftPaint()
        sync()
      })
    const ds = getDesignState()
    if (ds.projectThemes.length === 0 && ds.catalogThemes.length === 0) {
      void loadThemes().then(() => {
        if (gen !== openGen || openedForId !== id) return
        void run()
      })
    } else {
      void run()
    }
  }

  const leaveGuard = createLeaveGuard({
    isDirty: () => getDesignState().dirty,
    save: () => saveThemeForLeave(),
    discard: () => {
      discardThemeEdits()
      scheduleDraftPaint()
    },
    navigate: (href) => navigate(href),
    openTheme: (id) => {
      clearOpened()
      doOpenTheme(id)
    },
  })

  leaveGuard.subscribe(() => {
    const s = leaveGuard.snapshot()
    leaveOpen.set(s.open)
    leaveReason.set(s.reason)
  })

  const requestNavigate = (href: string, reason = 'Leave theme editor') => {
    // true = navigated; false = dialog open (caller must not also navigate)
    return leaveGuard.requestLeave({ href, openThemeId: null, reason })
  }

  // Shell activity rail → same leave dialog when dirty.
  setThemeLeaveNavigate((href) => requestNavigate(href, 'Leave theme editor'))

  const requestSwitchTheme = (id: string, reason = 'Switch theme') => {
    const ds = getDesignState()
    if (!ds.dirty || ds.themeId === id) {
      doOpenTheme(id)
      return
    }
    leaveGuard.requestLeave({ href: null, openThemeId: id, reason })
  }

  const onLeaveSave = () => {
    void leaveGuard.save().then((ok) => {
      if (ok) setNote(saveSuccessMessage, 'success')
      else {
        const err = getDesignState().lastWriteError
        if (err) setNote(err, 'destructive')
      }
      sync()
    })
  }
  const onLeaveDiscard = () => {
    void leaveGuard.discard()
    setNote('Discarded unsaved changes.')
    sync()
  }
  const onLeaveCancel = () => {
    leaveGuard.cancel()
    sync()
  }

  const syncSession = () => {
    const ds = getDesignState()
    const routeId = themeIdFromLocation() || ds.themeId || ''
    const id = ds.themeId || routeId
    themeId.set(id)
    const name = themeDisplayName(id)
    themeName.set(name)
    if (!nameEdit() || openedForId !== id) {
      nameEdit.set(name)
    }
    const resolved = resolvePaintId(id, ds.forkedFrom)
    paintId.set(resolved.paintId)
    if (resolved.warning && paintWarnForId !== id) {
      paintWarnForId = id
      setNote(resolved.warning, 'warning')
    }
    const info = modesForPaintId(resolved.paintId)
    const mode = resolvePaintMode(resolved.paintId)
    paintMode.set(mode)
    modesLabel.set(info.modesLabel)
    canToggleMode.set(info.dual)
    showAddMode.set(!info.dual)
    addModeLabel.set(info.modes[0] === 'dark' ? 'Add light table' : 'Add dark table')
    modeLightOn.set(mode === 'light')
    modeDarkOn.set(mode === 'dark')
    activeModeLabel.set(mode === 'dark' ? 'Dark' : 'Light')
    modesMeta.set(info.modesLabel)

    const editable = !!ds.themePath
    isCatalog.set(!editable)
    canEdit.set(editable)
    catalogHint.set(!editable)
    stateChipLabel.set(editable ? '' : 'Catalog')
    stateChipTone.set('muted')
    showStateChip.set(!editable)
    isDirty.set(ds.dirty)
    canUndo.set(!!ds.canUndo)
    canRedo.set(!!ds.canRedo)
    diffRows.set(
      (ds.diffRows ?? []).slice(0, 40).map((r, i) => ({
        key: `${r.path}-${i}`,
        path: r.path,
        from: String(r.from ?? '').slice(0, 48),
        to: String(r.to ?? '').slice(0, 48),
      })),
    )
    hasDiffRows.set((ds.diffRows ?? []).length > 0)
    pathCrumb.set(facetCrumb(facet))

    const blast = ds.blast
    usedByLabel.set(blast?.label ?? 'Not used in this project yet')
    saveReviewOpen.set(ds.saveReviewOpen)
    saveButtonLabel.set(ds.saveReviewOpen ? 'Confirm save' : 'Save')
    usedBySites.set(
      (blast?.sites ?? []).slice(0, 12).map((s, i) => ({
        file: s.file || '(unknown)',
        key: `${s.file || 'site'}-${i}`,
      })),
    )

    loading.set(ds.loading)
    const err = ds.error || ds.lastWriteError
    hasError.set(!!err)
    errorText.set(err ?? '')

    const lg = leaveGuard.snapshot()
    leaveOpen.set(lg.open)
    leaveReason.set(lg.reason)
  }

  /** Refresh Used by for a token (primary / radius-lg / …). */
  const refreshUsedBy = (token: string) => {
    void refreshBlast(token)
  }

  const facetUsageToken = (): string | null => {
    if (facet === 'color') return 'primary'
    if (facet === 'shape') return 'radius-lg'
    return null
  }

  const sync = () => {
    syncSession()
    notifyDomain()
  }

  const clearOpened = () => {
    openedForId = ''
  }

  const ensureThemeOpen = (id: string) => {
    if (!id) return
    if (openedForId === id && getDesignState().themeId === id && getDesignState().themeDoc) {
      sync()
      return
    }
    // Dirty switch to a different theme → leave guard.
    const ds = getDesignState()
    if (ds.dirty && ds.themeId && ds.themeId !== id) {
      requestSwitchTheme(id)
      return
    }
    doOpenTheme(id)
  }

  const facetHref = (f: ThemeFacet) => {
    const id = themeId() || themeIdFromLocation()
    return themeFacetHref(id, f)
  }

  const goBack = () => {
    requestNavigate(themesHomeHref(), 'Back to Themes')
  }

  const goOverview = () => {
    // Same draft — facet switch is free.
    navigate(facetHref('overview'))
  }

  const navigateFacet = (f: ThemeFacet) => {
    if (f === facet) {
      pathCrumb.set(facetCrumb(facet))
      return
    }
    // Same open draft across facets — no dirty guard.
    navigate(facetHref(f))
  }

  const onNameInput = (e: Event) => {
    nameEdit.set((e.target as HTMLInputElement).value)
  }

  const onNameChange = (e: Event) => {
    const v = (e.target as HTMLInputElement).value.trim()
    nameEdit.set(v || themeName())
    if (!renameNoteShown) {
      renameNoteShown = true
      setNote('Theme rename is not persisted yet — name is local to this session.', 'warning')
    }
  }

  const applyMode = (m: 'light' | 'dark') => {
    const pid = paintId()
    const info = modesForPaintId(pid)
    if (!info.modes.includes(m)) {
      setNote(`${themeName()} only ships a ${info.modesLabel.toLowerCase()} table.`)
      return
    }
    modeByPaintId.set(pid, m)
    setDesignMode(m)
    scheduleDraftPaint()
    sync()
  }

  /** Switch checked = dark mode; unchecked = light. */
  const onModeSwitch = (e: Event) => {
    const checked = (e.target as HTMLInputElement).checked
    applyMode(checked ? 'dark' : 'light')
  }

  const onAddMode = () => {
    setNote(
      'Mode tables are fixed by the theme file — authoring extra modes is not available.',
      'warning',
    )
  }

  const guardEdit = (): boolean => {
    if (canEdit()) return true
    setNote(editGuardMessage, 'warning')
    return false
  }

  /** Surfaces call this from specimen mount to register the preview host. */
  const attachPreviewHost = (el: HTMLElement | null | undefined) => {
    bindPreviewHost(el ?? null)
  }

  const shell = makeSessionShellActions({
    facet,
    saveSuccessMessage,
    themeId: () => themeId(),
    canEdit: () => canEdit(),
    setNote,
    sync,
    ensureThemeOpen,
    clearOpened,
    navigateFacet,
    pathCrumb,
    requestNavigate,
  })

  const writeMode = (): 'light' | 'dark' => {
    const pid = paintId()
    const info = modesForPaintId(pid)
    return info.dual ? paintMode() : info.defaultMode
  }

  syncSession()

  mountAmbient(() => {
    const unsub = subscribeDesign(() => sync())
    const ds = getDesignState()
    const boot = () => {
      const id = themeIdFromLocation()
      if (!id) {
        sync()
        return
      }
      if (ds.projectThemes.length === 0 && ds.catalogThemes.length === 0 && !ds.loading) {
        void loadThemes().then(() => ensureThemeOpen(id))
      } else {
        ensureThemeOpen(id)
      }
    }
    boot()
    // Dirty tab close / reload.
    const onBeforeUnload = (e: BeforeUnloadEvent) => {
      if (!getDesignState().dirty) return
      e.preventDefault()
      e.returnValue = ''
    }
    if (typeof window !== 'undefined') {
      // sail-allow sail-no-ambient-events
      window.addEventListener('beforeunload', onBeforeUnload)
    }
    return () => {
      unsub()
      leaveGuard.reset()
      setThemeLeaveNavigate(null)
      if (typeof window !== 'undefined') {
        // sail-allow sail-no-ambient-events
        window.removeEventListener('beforeunload', onBeforeUnload)
      }
    }
  })

  return {
    facet,
    themeId,
    themeName,
    nameEdit,
    isCatalog,
    canEdit,
    stateChipLabel,
    stateChipTone,
    showStateChip,
    modesLabel,
    canToggleMode,
    showAddMode,
    addModeLabel,
    modeLightOn,
    modeDarkOn,
    paintId,
    paintMode,
    activeModeLabel,
    isDirty,
    canUndo,
    canRedo,
    diffRows,
    hasDiffRows,
    pathCrumb,
    loading,
    hasError,
    errorText,
    catalogHint,
    modesMeta,
    usedByLabel,
    saveReviewOpen,
    saveButtonLabel,
    usedBySites,
    toastItems,
    leaveOpen,
    leaveReason,
    pathNav: () => PATH_NAV.map((p) => ({ ...p })),
    facetDoors: () => FACET_DOORS.map((p) => ({ ...p })),
    /**
     * Overflow menu removed from chrome (delete/remix on Themes home).
     * Keep a no-op iterator so stale HMR bundles that still call
     * `s().overflowNav()` do not crash the route boundary.
     */
    overflowNav: () => [] as { key: string; value: string; icon: string; label: string }[],
    pathMenuId: `theme-${facet}-path-menu`,
    chromeMarker: facet,
    goBack,
    goOverview,
    navigateFacet,
    facetHref,
    onNameInput,
    onNameChange,
    onModeSwitch,
    onAddMode,
    onSave: shell.onSave,
    onUndo: () => {
      if (undoThemeEdit()) {
        setNote('Undid last theme edit', 'muted')
        scheduleDraftPaint()
        sync()
      }
    },
    onRedo: () => {
      if (redoThemeEdit()) {
        setNote('Redid theme edit', 'muted')
        scheduleDraftPaint()
        sync()
      }
    },
    onPeek: shell.onPeek,
    onAddToProject: shell.onAddToProject,
    onPathSelect: shell.onPathSelect,
    onFacet: shell.onFacet,
    onRetry: shell.onRetry,
    onToastDismiss,
    onLeaveSave,
    onLeaveDiscard,
    onLeaveCancel,
    setNote,
    guardEdit,
    sync,
    onSync,
    ensureThemeOpen,
    writeMode,
    applyMode,
    refreshUsedBy,
    attachPreviewHost,
    requestNavigate,
  }
}
