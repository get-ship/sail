/**
 * Theme list / open / save / add-to-project via SailTheme ops.
 * Save + delete + add also run SailTheme/Generate mode=app so
 * registered app tables match disk for table consumers.
 */
import { validateMonotonic } from './scale'
import {
  CATALOG_THEME_IDS,
  isCatalogThemeId,
  parseThemeJson,
  projectThemesDir,
  setThemeId,
  stringifyTheme,
  themeFilesFromList,
  themeIdOf,
} from './theme-doc'
import type { DesignOpsClient, DesignSurfaceState, ThemeFile } from './types'
import { CATALOG_THEMES_DIR } from './types'
import { DEFAULT_GROUP_ID } from './token-groups'
import { userError } from '../user-error'

/** After project JSON write — regenerate app theme tables (not the edit-loop paint). */
async function generateAppThemes(
  ctx: ThemeIoCtx,
  noteOnFail = true,
): Promise<void> {
  const client = ctx.getClient()
  const state = ctx.getState()
  if (!client?.sailThemeGenerate) {
    // Honest gap: ops client without Generate (tests / offline). JSON still saved.
    if (noteOnFail) {
      ctx.setState({
        ...ctx.getState(),
        statusNote:
          (ctx.getState().statusNote || 'Theme saved') +
          ' · tables not regenerated (Generate unavailable)',
      })
    }
    return
  }
  try {
    await client.sailThemeGenerate({
      mode: 'app',
      ...(state.projectRoot ? { project: state.projectRoot } : {}),
    })
  } catch (e) {
    if (noteOnFail) {
      ctx.setState({
        ...ctx.getState(),
        lastWriteError: userError(
          e,
          'Theme JSON saved, but app theme tables failed to regenerate.',
        ),
      })
      ctx.emit()
    }
  }
}

export type ThemeIoCtx = {
  getClient: () => DesignOpsClient | null
  getState: () => DesignSurfaceState
  setState: (s: DesignSurfaceState) => void
  getSavedDoc: () => Record<string, unknown> | null
  setSavedDoc: (d: Record<string, unknown> | null) => void
  recomputeDerived: (doc: Record<string, unknown> | null) => void
  emit: () => void
  repaintSpecimen: () => Promise<void>
  refreshBlast: (token: string) => Promise<void>
}

export function setDesignProjectRootOn(
  ctx: ThemeIoCtx,
  root: string,
): void {
  const themesDir = projectThemesDir(root || 'apps/studio')
  ctx.setState({ ...ctx.getState(), projectRoot: root, themesDir })
  ctx.emit()
}

export async function loadThemesOn(
  ctx: ThemeIoCtx,
  opts?: { themeId?: string },
): Promise<void> {
  const client = ctx.getClient()
  let state = ctx.getState()
  if (!client) {
    // Not an error — chrome may still be wiring ops. Surfaces show empty, not red.
    ctx.setState({ ...state, error: null, loading: false })
    ctx.emit()
    return
  }
  ctx.setState({ ...state, loading: true, error: null })
  ctx.emit()
  state = ctx.getState()
  try {
    const [cat, proj, registry] = await Promise.all([
      client.sailThemeList({ path: CATALOG_THEMES_DIR }),
      client.sailThemeList({ path: state.themesDir }).catch(() => ({ files: [] as string[] })),
      client.sailThemeRegistry?.({ section: 'catalog' }).catch(() => null),
    ])
    const catalogOrder =
      registry?.catalog?.filter((id): id is string => typeof id === 'string' && isCatalogThemeId(id))
        ?? [...CATALOG_THEME_IDS]
    const catalogRank = new Map(catalogOrder.map((id, i) => [id, i]))
    const catalogThemes = themeFilesFromList(
      cat.files ?? [],
      CATALOG_THEMES_DIR,
      true,
    )
      .filter((t) => catalogRank.has(t.id))
      .sort((a, b) => (catalogRank.get(a.id) ?? 99) - (catalogRank.get(b.id) ?? 99))
    const projectThemes = themeFilesFromList(
      proj.files ?? [],
      state.themesDir,
      false,
    )
    state = ctx.getState()
    // Dirty catalog refresh: update lists only — no sailThemeRead / openTheme
    // so the working doc is preserved (and tests can assert zero reads).
    if (state.dirty && state.themeDoc && !opts?.themeId) {
      ctx.setState({
        ...ctx.getState(),
        catalogThemes,
        projectThemes,
        loading: false,
      })
      ctx.emit()
      return
    }
    // Hydrate project mode tables from DTCG so gallery swatches paint with the
    // correct mode without relying on package themes-meta contamination.
    const projectWithModes = await Promise.all(
      projectThemes.map(async (t) => {
        try {
          const r = await client.sailThemeRead({ path: t.path })
          const doc = parseThemeJson(r.content)
          const modesObj = doc.modes
          const modes =
            modesObj && typeof modesObj === 'object' && !Array.isArray(modesObj)
              ? Object.keys(modesObj as Record<string, unknown>).filter(
                  (m): m is 'light' | 'dark' => m === 'light' || m === 'dark',
                )
              : []
          const ext = doc.$extensions as Record<string, unknown> | undefined
          const ship = ext?.sail as Record<string, unknown> | undefined
          const dmRaw = ship?.defaultMode
          const defaultMode =
            dmRaw === 'light' || dmRaw === 'dark' ? dmRaw : undefined
          if (modes.length || defaultMode) {
            return {
              ...t,
              ...(modes.length ? { modes } : {}),
              ...(defaultMode ? { defaultMode } : {}),
            }
          }
        } catch {
          /* list still works; paint falls back to modesForPaintId */
        }
        return t
      }),
    )
    ctx.setState({
      ...ctx.getState(),
      catalogThemes,
      projectThemes: projectWithModes,
    })
    state = ctx.getState()
    const prefer =
      opts?.themeId ||
      state.themeId ||
      projectThemes[0]?.id ||
      catalogThemes.find((t) => t.id === 'sail')?.id ||
      catalogThemes[0]?.id ||
      'sail'
    if (!opts?.themeId && state.themeDoc && state.themeId === prefer) {
      ctx.setState({ ...ctx.getState(), loading: false })
      ctx.emit()
      return
    }
    await openThemeOn(ctx, prefer)
  } catch (e) {
    ctx.setState({
      ...ctx.getState(),
      loading: false,
      error: userError(e, 'Could not load themes — try again.'),
    })
    ctx.emit()
  }
}

export async function openThemeOn(ctx: ThemeIoCtx, themeId: string): Promise<void> {
  const client = ctx.getClient()
  if (!client) return
  ctx.setState({ ...ctx.getState(), loading: true, error: null })
  ctx.emit()
  try {
    const state = ctx.getState()
    const proj = state.projectThemes.find((t) => t.id === themeId)
    const cat = state.catalogThemes.find((t) => t.id === themeId)
    const file: ThemeFile | undefined = proj ?? cat
    if (!file) {
      const path = `${CATALOG_THEMES_DIR}/${themeId}.json`
      const r = await client.sailThemeRead({ path })
      const doc = parseThemeJson(r.content)
      ctx.setSavedDoc(structuredClone(doc))
      ctx.setState({
        ...ctx.getState(),
        themeId: themeIdOf(doc, themeId),
        forkedFrom: null,
        themePath: null,
        loading: false,
      })
      ctx.recomputeDerived(doc)
      ctx.emit()
      await ctx.repaintSpecimen()
      return
    }
    const r = await client.sailThemeRead({ path: file.path })
    const doc = parseThemeJson(r.content)
    ctx.setSavedDoc(structuredClone(doc))
    ctx.setState({
      ...ctx.getState(),
      themeId: themeIdOf(doc, file.id),
      forkedFrom: file.catalog ? null : ctx.getState().forkedFrom,
      themePath: file.catalog ? null : file.path,
      loading: false,
    })
    ctx.recomputeDerived(doc)
    ctx.emit()
    await ctx.repaintSpecimen()
    if (ctx.getState().groupId === 'radius') await ctx.refreshBlast('radius-lg')
  } catch (e) {
    ctx.setState({
      ...ctx.getState(),
      loading: false,
      error: userError(e, 'Could not open that theme — try again.'),
    })
    ctx.emit()
  }
}

/**
 * Add a catalog (or project) theme into the project as `newId`.
 * Product UI: "Add to project" — never "fork".
 */
export async function forkThemeOn(
  ctx: ThemeIoCtx,
  newId: string,
  sourceId?: string,
): Promise<boolean> {
  const client = ctx.getClient()
  if (!client) {
    ctx.setState({ ...ctx.getState(), lastWriteError: 'Could not save themes right now' })
    ctx.emit()
    return false
  }
  const id = newId.trim()
  if (!/^[a-z][a-z0-9-]*$/.test(id)) {
    ctx.setState({
      ...ctx.getState(),
      lastWriteError: 'Use lowercase letters, numbers, and hyphens',
    })
    ctx.emit()
    return false
  }
  const state = ctx.getState()
  // Clobber risk: refuse silent overwrite of an existing project theme.
  if (state.projectThemes.some((t) => t.id === id)) {
    ctx.setState({
      ...ctx.getState(),
      lastWriteError: `Project already has a theme named “${id}”.`,
    })
    ctx.emit()
    return false
  }
  const srcId = sourceId || state.themeId
  const src =
    state.projectThemes.find((t) => t.id === srcId) ||
    state.catalogThemes.find((t) => t.id === srcId)
  const source_path = src?.path || `${CATALOG_THEMES_DIR}/${srcId}.json`
  const dest_path = `${state.themesDir}/${id}.json`
  try {
    let content: string | undefined
    if (state.themeDoc && (!sourceId || sourceId === state.themeId)) {
      content = stringifyTheme(setThemeId(state.themeDoc, id))
    }
    await client.sailThemeFork({
      source_path,
      dest_path,
      new_id: id,
      ...(content !== undefined ? { content } : {}),
    })
    ctx.setState({
      ...ctx.getState(),
      forkedFrom: srcId,
      themePath: dest_path,
      themeId: id,
      lastWriteError: null,
    })
    await generateAppThemes(ctx, false)
    await loadThemesOn(ctx, { themeId: id })
    return true
  } catch (e) {
    ctx.setState({
      ...ctx.getState(),
      lastWriteError: userError(e, 'Could not add that theme to the project — try again.'),
    })
    ctx.emit()
    return false
  }
}

export async function saveThemeOn(ctx: ThemeIoCtx): Promise<boolean> {
  const client = ctx.getClient()
  const state = ctx.getState()
  if (!client || !state.themeDoc) {
    ctx.setState({ ...state, lastWriteError: 'Nothing to save' })
    ctx.emit()
    return false
  }
  let path = state.themePath
  if (!path) {
    const newId =
      state.themeId.startsWith('my-') ? state.themeId : `my-${state.themeId}`
    const ok = await forkThemeOn(ctx, newId, state.themeId)
    if (!ok) return false
    path = ctx.getState().themePath
  }
  if (!path) {
    ctx.setState({ ...ctx.getState(), lastWriteError: "Can't save this theme right now" })
    ctx.emit()
    return false
  }
  const s = ctx.getState()
  const monoErr = validateMonotonic(s.radius.steps, s.radius.ordered, s.radius.path)
  if (monoErr) {
    ctx.setState({
      ...ctx.getState(),
      lastWriteError: 'Radius steps must go from small to large',
    })
    ctx.emit()
    return false
  }
  try {
    const content = stringifyTheme(s.themeDoc!)
    await client.sailThemeWrite({ path, content })
    ctx.setSavedDoc(structuredClone(s.themeDoc))
    ctx.setState({
      ...ctx.getState(),
      dirty: false,
      lastWriteError: null,
      diffRows: [],
      themePath: path,
      statusNote: 'Theme saved',
    })
    ctx.emit()
    // Registered CSS tables for attr-only / non-apply consumers.
    await generateAppThemes(ctx, true)
    await ctx.repaintSpecimen()
    return true
  } catch (e) {
    ctx.setState({
      ...ctx.getState(),
      lastWriteError: userError(e, 'Could not save the theme — try again.'),
    })
    ctx.emit()
    return false
  }
}

/** Delete a project theme file (catalog themes refused by ops). */
export async function deleteThemeOn(
  ctx: ThemeIoCtx,
  themeId: string,
): Promise<boolean> {
  const client = ctx.getClient()
  const state = ctx.getState()
  if (!client?.sailThemeDelete) {
    ctx.setState({
      ...state,
      lastWriteError: 'Theme delete is not available offline.',
    })
    ctx.emit()
    return false
  }
  const row =
    state.projectThemes.find((t) => t.id === themeId) ||
    (state.themeId === themeId && state.themePath
      ? { id: themeId, path: state.themePath, file: '', catalog: false }
      : null)
  if (!row?.path || row.catalog) {
    ctx.setState({
      ...state,
      lastWriteError: 'Only project themes can be deleted.',
    })
    ctx.emit()
    return false
  }
  try {
    await client.sailThemeDelete({ path: row.path })
    // Clear active if we deleted the open theme.
    if (state.themeId === themeId) {
      ctx.setSavedDoc(null)
      ctx.setState({
        ...ctx.getState(),
        themeId: '',
        themePath: null,
        themeDoc: null,
        dirty: false,
        diffRows: [],
        lastWriteError: null,
        statusNote: 'Theme deleted',
      })
    } else {
      ctx.setState({
        ...ctx.getState(),
        lastWriteError: null,
        statusNote: 'Theme deleted',
      })
    }
    ctx.emit()
    await generateAppThemes(ctx, false)
    await loadThemesOn(ctx)
    return true
  } catch (e) {
    ctx.setState({
      ...ctx.getState(),
      lastWriteError: userError(e, 'Could not delete the theme — try again.'),
    })
    ctx.emit()
    return false
  }
}

/** Engine-gated generate: check_theme then write+tokens generate. */
export async function generateThemeOn(ctx: ThemeIoCtx): Promise<boolean> {
  const client = ctx.getClient()
  const state = ctx.getState()
  if (!client?.sailThemeCheckTheme || !state.themeDoc) {
    ctx.setState({
      ...state,
      lastWriteError: 'Generate needs the theme engine.',
    })
    ctx.emit()
    return false
  }
  try {
    const check = await client.sailThemeCheckTheme({
      theme: state.themeDoc,
      derive: true,
      label: state.themeId || 'theme',
    })
    if (!check.ok) {
      const err =
        (Array.isArray(check.errors) && check.errors[0]) ||
        'Theme failed the contrast gate.'
      ctx.setState({
        ...ctx.getState(),
        lastWriteError: String(err),
        statusNote: 'Generate refused — contrast gate',
      })
      ctx.emit()
      return false
    }
    if (!ctx.getState().themePath) {
      ctx.setState({
        ...ctx.getState(),
        lastWriteError: null,
        statusNote: 'Theme is valid. Add to project to save.',
      })
      ctx.emit()
      return true
    }
    return saveThemeOn(ctx)
  } catch (e) {
    ctx.setState({
      ...ctx.getState(),
      lastWriteError: userError(e, 'Could not generate the theme — try again.'),
    })
    ctx.emit()
    return false
  }
}

export function emptyDesignState(): DesignSurfaceState {
  return {
    projectRoot: '',
    themesDir: 'apps/studio/themes',
    groupId: DEFAULT_GROUP_ID,
    decisionId: 'color',
    density: 'default',
    themeId: 'sail',
    modeId: 'dark',
    forkedFrom: null,
    themePath: null,
    themeDoc: null,
    dirty: false,
    saveReviewOpen: false,
    catalogThemes: [],
    projectThemes: [],
    radius: {
      path: 'radius',
      ordered: ['sm', 'md', 'lg', 'xl'],
      scaleTogether: true,
      steps: [
        { key: 'sm', valuePx: 4, raw: '4px' },
        { key: 'md', valuePx: 6, raw: '6px' },
        { key: 'lg', valuePx: 10, raw: '10px' },
        { key: 'xl', valuePx: 14, raw: '14px' },
      ],
    },
    inheritance: [],
    // Studio workbench: display/serif (Fraunces) for heroes; UI sans for body.
    typeRoles: { display: 'Fraunces', body: 'DM Sans', mono: 'JetBrains Mono' },
    icons: { pack: 'lucide', glyphs: [], query: '' },
    iconScale: {
      path: 'icon',
      ordered: ['sm', 'md', 'lg', 'xl'],
      scaleTogether: true,
      steps: [
        { key: 'sm', valuePx: 14, raw: '14px' },
        { key: 'md', valuePx: 16, raw: '16px' },
        { key: 'lg', valuePx: 20, raw: '20px' },
        { key: 'xl', valuePx: 24, raw: '24px' },
      ],
    },
    blast: null,
    inspectOpen: false,
    overridesOnly: false,
    tokenSearch: '',
    overrideCount: 0,
    specimenPng: null,
    specimenMs: null,
    loading: false,
    error: null,
    lastWriteError: null,
    lastClampNote: null,
    diffRows: [],
    statusNote: 'Pick a decision or token group',
    canUndo: false,
    canRedo: false,
  }
}
