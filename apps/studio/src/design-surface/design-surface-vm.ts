/**
 * Design surface VM — five decisions + token power mode + theme chrome.
 * Shell owns loadThemes + ops client.
 */
import { mountAmbient, signal } from '@ship/sail-runtime'
import {
  clearTokenValue,
  exportDiffText,
  formatDiffLine,
  getDesignState,
  loadThemes,
  openTheme,
  previewInApp,
  requestSaveTheme,
  setBrandColor,
  setDecision,
  setOverridesOnly,
  setSaveReviewOpen,
  setTokenGroup,
  setTokenSearch,
  setTokenValue,
  subscribeDesign,
  uiActiveGroup,
  uiBrand,
  uiDecisions,
  uiGroupTokens,
  uiGroups,
  uiHeader,
  uiThemeOptions,
  type DecisionId,
} from '@ship/studio/design'

type GroupUi = {
  id: string
  label: string
  count: string
  active: boolean
  key: string
  tone: 'primary' | 'default'
}
type ThemeOptUi = { id: string; label: string; key: string }
type TokenRowUi = {
  path: string
  label: string
  cssName: string
  value: string
  display: string
  dirty: boolean
  isColor: boolean
  isDim: boolean
  isText: boolean
  num: number
  unit: string
  min: number
  max: number
  step: number
  key: string
  groupLabel: string
  groupId: string
  showGroup: boolean
  colorEditOpen: boolean
  pickerHex: string
}
type DiffUi = {
  path: string
  line: string
  key: string
  label: string
  fromDisplay: string
  toDisplay: string
  isColor: boolean
  toValue: string
}
type DecisionUi = {
  id: string
  label: string
  summary: string
  active: boolean
  key: string
  tone: 'primary' | 'default'
  isColor: boolean
  swatch: string
}
type RampUi = { key: string; label: string; value: string; hex: string }
type ContrastUi = {
  id: string
  label: string
  ratio: string
  passAA: boolean
  key: string
  tone: 'success' | 'destructive'
}

export function createDesignSurface() {
  const groupRows = signal<GroupUi[]>([])
  const decisionRows = signal<DecisionUi[]>([])
  const tokenRows = signal<TokenRowUi[]>([])
  const hasTokenRows = signal(false)
  const designThemeOpts = signal<ThemeOptUi[]>([])
  const designThemeId = signal('studio')
  const designForkLabel = signal('')
  const designOverridesLabel = signal('0 changes')
  const designThemeLabel = signal('ship')
  const activeGroupLabel = signal('Color')
  const activeGroupId = signal('color')
  const tokenSearchVal = signal('')
  const isSearching = signal(false)
  const overridesOnlyOn = signal(false)
  const designStatusNote = signal('Pick a decision or token group')
  const designError = signal('')
  const hasDesignError = signal(false)
  const designWriteError = signal('')
  const hasDesignWriteError = signal(false)
  const clampNote = signal('')
  const hasClampNote = signal(false)
  const diffRows = signal<DiffUi[]>([])
  const hasDiff = signal(false)
  const tokenCountLabel = signal('')
  const colorEditPath = signal('')
  const brandHex = signal('#e07a3d')
  const brandRamp = signal<RampUi[]>([])
  const contrastRows = signal<ContrastUi[]>([])
  const hasContrastFail = signal(false)
  const contrastFailLabel = signal('')
  const isBrandDecision = signal(true)
  const saveReviewOpen = signal(false)
  const blastLabel = signal('')
  const saveButtonLabel = signal('Save theme')
  const canConfirmSave = signal(false)

  const sync = () => {
    const ds = getDesignState()
    const searching = ds.tokenSearch.trim().length > 0
    groupRows.set(
      uiGroups().map((g) => ({
        ...g,
        tone: g.active ? 'primary' : 'default',
      })),
    )
    decisionRows.set(
      uiDecisions().map((d) => ({
        id: d.id,
        label: d.label,
        summary: d.summary,
        active: d.active,
        key: d.key,
        tone: d.tone,
        isColor: d.isColor,
        swatch: d.swatch,
      })),
    )
    const editPath = colorEditPath()
    const tokens = uiGroupTokens().map((t) => ({
      ...t,
      showGroup: searching && t.groupId !== ds.groupId,
      colorEditOpen: t.isColor && t.path === editPath,
      pickerHex: t.pickerHex ?? '',
    }))
    tokenRows.set(tokens)
    hasTokenRows.set(tokens.length > 0)
    designThemeOpts.set(
      uiThemeOptions().map((t) => ({ id: t.id, label: t.label, key: t.id })),
    )
    const h = uiHeader()
    const g = uiActiveGroup()
    designThemeId.set(ds.themeId)
    designThemeLabel.set(h.themeLabel)
    designForkLabel.set(h.forkLabel)
    designOverridesLabel.set(h.overridesLabel)
    activeGroupLabel.set(searching ? 'Search results' : g.label)
    activeGroupId.set(g.id)
    tokenSearchVal.set(ds.tokenSearch)
    isSearching.set(searching)
    overridesOnlyOn.set(ds.overridesOnly)
    designStatusNote.set(ds.statusNote)
    designError.set(ds.error ?? '')
    hasDesignError.set(!!ds.error)
    designWriteError.set(ds.lastWriteError ?? '')
    hasDesignWriteError.set(!!ds.lastWriteError)
    clampNote.set(ds.lastClampNote ?? '')
    hasClampNote.set(!!ds.lastClampNote)
    const brand = uiBrand()
    brandHex.set(brand.primaryHex)
    brandRamp.set(brand.ramp)
    contrastRows.set(
      brand.contrast.map((c) => ({
        id: c.id,
        label: c.label,
        ratio: `${c.ratio}:1`,
        passAA: c.passAA,
        key: c.id,
        tone: c.passAA ? 'success' : 'destructive',
      })),
    )
    hasContrastFail.set(brand.contrastFailCount > 0)
    contrastFailLabel.set(
      brand.contrastFailCount > 0
        ? `${brand.contrastFailCount} contrast check${brand.contrastFailCount === 1 ? '' : 's'} below AA`
        : 'All specimen pairs meet AA',
    )
    isBrandDecision.set(ds.decisionId === 'color' || g.isColor)
    saveReviewOpen.set(ds.saveReviewOpen)
    blastLabel.set(ds.blast?.label ?? 'Review where this theme is used')
    saveButtonLabel.set(ds.saveReviewOpen ? 'Confirm save' : 'Save theme')
    canConfirmSave.set(ds.dirty)
    diffRows.set(
      ds.diffRows.map((r) => {
        const line = formatDiffLine(r)
        const isColor =
          r.to.startsWith('oklch') ||
          r.to.startsWith('#') ||
          r.to.startsWith('rgb') ||
          r.from.startsWith('oklch')
        const parts = line.split(': ')
        const label = parts[0] ?? r.path
        const arrow = parts.slice(1).join(': ')
        const [fromDisplay, toDisplay] = arrow.includes(' → ')
          ? arrow.split(' → ')
          : [r.from, r.to]
        return {
          path: r.path,
          line,
          key: r.path,
          label,
          fromDisplay: fromDisplay ?? '',
          toDisplay: toDisplay ?? '',
          isColor,
          toValue: r.to,
        }
      }),
    )
    hasDiff.set(ds.diffRows.length > 0)
    tokenCountLabel.set(
      searching
        ? `${tokens.length} match${tokens.length === 1 ? '' : 'es'}`
        : `${tokens.length} tokens`,
    )
  }

  sync()

  mountAmbient(() => {
    const unsub = subscribeDesign(() => sync())
    return () => unsub()
  })

  const onRetryDesign = () => {
    void loadThemes().then(() => sync())
  }

  const onDecisionClick = (e: Event) => {
    const t = (e.target as Element | null)?.closest?.('[data-decision]') as HTMLElement | null
    if (!t?.dataset.decision) return
    setDecision(t.dataset.decision as DecisionId)
    sync()
  }

  const onGroupClick = (e: Event) => {
    const t = (e.target as Element | null)?.closest?.('[data-group]') as HTMLElement | null
    if (!t?.dataset.group) return
    setTokenGroup(t.dataset.group)
    sync()
  }

  const onDesignTheme = (e: Event) => {
    const id = (e.target as HTMLSelectElement).value
    void openTheme(id).then(() => sync())
  }

  const onSaveTheme = () => {
    void requestSaveTheme().then(() => sync())
  }

  const onCancelSaveReview = () => {
    setSaveReviewOpen(false)
    sync()
  }

  const onExportDiff = () => {
    const text = exportDiffText()
    if (typeof navigator !== 'undefined' && navigator.clipboard?.writeText) {
      void navigator.clipboard.writeText(text)
    }
    sync()
  }

  const onPreviewInApp = () => {
    previewInApp()
    sync()
  }

  const onTokenSearch = (e: Event) => {
    setTokenSearch((e.target as HTMLInputElement).value)
    sync()
  }

  const onOverridesOnly = (e: Event) => {
    setOverridesOnly((e.target as HTMLInputElement).checked)
    sync()
  }

  const onTokenText = (e: Event) => {
    const t = e.target as HTMLInputElement
    const path = t.dataset.tokenPath
    if (!path) return
    void setTokenValue(path, t.value).then(() => sync())
  }

  const onTokenNum = (e: Event) => {
    const t = e.target as HTMLInputElement
    const path = t.dataset.tokenPath
    if (!path) return
    const unit = t.dataset.tokenUnit || ''
    const n = Number(t.value)
    if (Number.isNaN(n)) return
    void setTokenValue(path, `${n}${unit}`).then(() => sync())
  }

  const onTokenReset = (e: Event) => {
    const t = (e.target as Element | null)?.closest?.('[data-token-path]') as HTMLElement | null
    const path = t?.dataset.tokenPath
    if (!path) return
    void clearTokenValue(path).then(() => sync())
  }

  const onColorEditToggle = (e: Event) => {
    const t = (e.target as Element | null)?.closest?.('[data-token-path]') as HTMLElement | null
    const path = t?.dataset.tokenPath
    if (!path) return
    colorEditPath.set(colorEditPath() === path ? '' : path)
    sync()
  }

  const onTokenColor = (e: Event) => {
    const ce = e as CustomEvent<{ value?: string }>
    const host = e.currentTarget as HTMLElement | null
    const path = host?.dataset?.tokenPath
    const hex = ce.detail?.value ?? (host as HTMLElement & { value?: string })?.value
    if (!path || !hex) return
    void setTokenValue(path, hex).then(() => sync())
  }

  const onBrandColor = (e: Event) => {
    const ce = e as CustomEvent<{ value?: string }>
    const host = e.currentTarget as HTMLElement & { value?: string }
    const hex = ce.detail?.value ?? host?.value
    if (!hex) return
    void setBrandColor(hex).then(() => sync())
  }

  const onTokenRowActivate = (e: Event) => {
    const t = (e.target as Element | null)?.closest?.('[data-token-group]') as HTMLElement | null
    const gid = t?.dataset.tokenGroup
    if (!gid) return
    if (getDesignState().groupId !== gid) {
      setTokenGroup(gid)
      sync()
    }
  }

  const paintSwatches = () => {
    if (typeof document === 'undefined') return
    for (const el of document.querySelectorAll<HTMLElement>('[data-swatch]')) {
      const v = el.dataset.swatch
      if (v) el.style.background = v
    }
  }

  mountAmbient(() => {
    const id = setInterval(paintSwatches, 200)
    paintSwatches()
    return () => clearInterval(id)
  })

  return {
    groupRows,
    decisionRows,
    tokenRows,
    hasTokenRows,
    designThemeOpts,
    designThemeId,
    designForkLabel,
    designOverridesLabel,
    designThemeLabel,
    activeGroupLabel,
    activeGroupId,
    tokenSearchVal,
    isSearching,
    overridesOnlyOn,
    designStatusNote,
    designError,
    hasDesignError,
    designWriteError,
    hasDesignWriteError,
    clampNote,
    hasClampNote,
    diffRows,
    hasDiff,
    tokenCountLabel,
    brandHex,
    brandRamp,
    contrastRows,
    hasContrastFail,
    contrastFailLabel,
    isBrandDecision,
    saveReviewOpen,
    blastLabel,
    saveButtonLabel,
    canConfirmSave,
    onDecisionClick,
    onGroupClick,
    onRetryDesign,
    onDesignTheme,
    onSaveTheme,
    onCancelSaveReview,
    onExportDiff,
    onPreviewInApp,
    onTokenSearch,
    onOverridesOnly,
    onTokenText,
    onTokenNum,
    onTokenReset,
    onColorEditToggle,
    onTokenColor,
    onBrandColor,
    onTokenRowActivate,
  }
}
