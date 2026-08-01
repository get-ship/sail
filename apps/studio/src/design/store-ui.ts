/**
 * Design store UI projections + font/icon helpers.
 */
import type { DesignOpsClient, DesignSurfaceState } from './types'
import {
  densityLabel,
  formatDiffLine,
  uiActiveGroupMeta,
  uiBrandPanel,
  uiDecisionCards,
  uiGroupTokenRows,
  uiHeader as uiHeaderFrom,
  uiIconScaleSteps as uiIconScaleStepsFrom,
  uiInheritance as uiInheritanceFrom,
  uiRadiusSteps as uiRadiusStepsFrom,
  uiThemeOptions as uiThemeOptionsFrom,
  uiTokenGroups,
  uiTokenRowsLabeled,
} from './ui'
import { userError } from '../user-error'

export type StoreUiAccess = {
  getState: () => DesignSurfaceState
  getSaved: () => Record<string, unknown> | null
  getClient: () => DesignOpsClient | null
  setState: (s: DesignSurfaceState) => void
  emit: () => void
  setTypeRole: (role: 'display' | 'body' | 'mono', family: string) => Promise<void>
}

export function makeStoreUi(access: StoreUiAccess) {
  const uiGroups = () => uiTokenGroups(access.getState())
  const uiActiveGroup = () => uiActiveGroupMeta(access.getState())
  const uiGroupTokens = () => uiGroupTokenRows(access.getState(), access.getSaved())
  const uiRadiusSteps = () => uiRadiusStepsFrom(access.getState())
  const uiInheritance = () => uiInheritanceFrom(access.getState())
  const uiIconScaleSteps = () => uiIconScaleStepsFrom(access.getState())
  const uiThemeOptions = () => uiThemeOptionsFrom(access.getState())
  const uiHeader = () => uiHeaderFrom(access.getState())
  const uiDecisions = () => uiDecisionCards(access.getState())
  const uiBrand = () => uiBrandPanel(access.getState())
  const uiDensityLabel = () => densityLabel(access.getState().density)

  const uiTokenRows = () =>
    uiTokenRowsLabeled(access.getState(), access.getSaved()).map((r) => ({
      path: r.path,
      value: r.value,
      dirty: r.dirty,
      key: r.path,
      label: r.label,
      display: r.display,
      isColor: r.isColor,
      pickerHex: r.pickerHex,
    }))

  const exportDiffText = () => {
    const rows = access.getState().diffRows
    if (!rows.length) return 'No edits\n'
    return rows.map((r) => formatDiffLine(r)).join('\n') + '\n'
  }

  const filteredIconGlyphs = () => {
    const s = access.getState()
    const q = s.icons.query.trim().toLowerCase()
    const all = s.icons.glyphs
    if (!q) return all.slice(0, 80)
    return all.filter((g) => g.includes(q)).slice(0, 80)
  }

  const searchFonts = async (query: string): Promise<string[]> => {
    const client = access.getClient()
    if (!client?.sailThemeFontSearch) return []
    try {
      const r = await client.sailThemeFontSearch({ query })
      return r.families ?? []
    } catch {
      return []
    }
  }

  const addFontFamily = async (family: string) => {
    const client = access.getClient()
    if (client?.sailThemeFontAdd) {
      try {
        await client.sailThemeFontAdd({ family })
        await access.setTypeRole('body', family)
        access.setState({
          ...access.getState(),
          statusNote: `Added font ${family}`,
        })
        access.emit()
        return { ok: true, message: `Added ${family}` }
      } catch (e) {
        return {
          ok: false,
          message: userError(e, `Could not install ${family} — try again.`),
        }
      }
    }
    await access.setTypeRole('body', family)
    return {
      ok: false,
      message: `Could not install the font; body text set to ${family}`,
    }
  }

  const loadIconGlyphs = async (names?: string[]) => {
    const glyphs =
      names && names.length > 0 ? names : await fetchIconRegistryNames()
    access.setState({
      ...access.getState(),
      icons: { ...access.getState().icons, glyphs },
    })
    access.emit()
  }

  return {
    uiGroups,
    uiActiveGroup,
    uiGroupTokens,
    uiRadiusSteps,
    uiInheritance,
    uiIconScaleSteps,
    uiThemeOptions,
    uiHeader,
    uiDecisions,
    uiBrand,
    uiDensityLabel,
    uiTokenRows,
    exportDiffText,
    filteredIconGlyphs,
    searchFonts,
    addFontFamily,
    loadIconGlyphs,
  }
}

async function fetchIconRegistryNames(): Promise<string[]> {
  // Prefer build-used list when the host serves it; registry.json is always on
  // `/_deck/icons/*`. Avoid requesting a known-missing path first — browser
  // network logs treated the 404 as a studio failure even though we fell back.
  const used = await fetchJsonNameList('/_deck/icons/registry.json')
  return used
}

async function fetchJsonNameList(url: string): Promise<string[]> {
  try {
    const r = await fetch(url)
    if (!r.ok) return []
    const j = (await r.json()) as unknown
    if (Array.isArray(j)) return j.map(String).filter(Boolean)
  } catch {
    /* offline */
  }
  return []
}

