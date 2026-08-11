/**
 * Design store UI projections + font/icon helpers.
 */
import { query } from '../../../../packages/sail/runtime/src/index.ts'
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

export type FontSearchHit = {
  family: string
  category: string
  popularity: number
  mono: boolean
}

export type FontSearchResult = {
  families: string[]
  hits: FontSearchHit[]
  count: number
  catalogSize: number
  source: string
}

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

  const searchFonts = async (
    query: string,
    opts?: { category?: string; limit?: number },
  ): Promise<FontSearchResult> => {
    const empty: FontSearchResult = {
      families: [],
      hits: [],
      count: 0,
      catalogSize: 0,
      source: 'none',
    }
    const client = access.getClient()
    if (!client?.sailThemeFontSearch) return empty
    try {
      const r = await client.sailThemeFontSearch({
        query,
        category: opts?.category,
        limit: opts?.limit ?? 40,
      })
      const families = (r.families ?? []).filter(Boolean)
      const hits: FontSearchHit[] = (r.hits ?? [])
        .map((h) => ({
          family: String(h.family ?? ''),
          category: String(h.category ?? 'Other'),
          popularity: Number(h.popularity ?? 99_999),
          mono: Boolean(h.mono),
        }))
        .filter((h) => h.family)
      // Back-compat: if server only returns families, synthesize hits.
      const resolvedHits =
        hits.length > 0
          ? hits
          : families.map((family, i) => ({
              family,
              category: 'Other',
              popularity: i,
              mono: /mono|code/i.test(family),
            }))
      return {
        families: families.length ? families : resolvedHits.map((h) => h.family),
        hits: resolvedHits,
        count: Number(r.count ?? resolvedHits.length),
        catalogSize: Number(r.catalogSize ?? 0),
        source: String(r.source ?? 'unknown'),
      }
    } catch {
      return empty
    }
  }

  const addFontFamily = async (
    family: string,
    role: 'display' | 'body' | 'mono' = 'body',
  ) => {
    const client = access.getClient()
    if (client?.sailThemeFontAdd) {
      try {
        await client.sailThemeFontAdd({ family })
        await access.setTypeRole(role, family)
        access.setState({
          ...access.getState(),
          statusNote: `Installed ${family} → ${role}`,
        })
        access.emit()
        return { ok: true, message: `Installed ${family} and set ${role}` }
      } catch (e) {
        return {
          ok: false,
          message: userError(e, `Could not install ${family} — try again.`),
        }
      }
    }
    await access.setTypeRole(role, family)
    return {
      ok: false,
      message: `Ops FontAdd unavailable; ${role} set to ${family} (not vendored)`,
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

async function fetchJsonNameList(_url: string): Promise<string[]> {
  // DOMDEC: ambient fetch banned — icon name list needs query() inject (gap).
  return []
}

