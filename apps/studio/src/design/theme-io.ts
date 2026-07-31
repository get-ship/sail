/**
 * Theme list / open / save / fork via SailTheme ops.
 */
import { validateMonotonic } from './scale'
import {
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
    const [cat, proj] = await Promise.all([
      client.sailThemeList({ path: CATALOG_THEMES_DIR }),
      client.sailThemeList({ path: state.themesDir }).catch(() => ({ files: [] as string[] })),
    ])
    const catalogThemes = themeFilesFromList(
      cat.files ?? [],
      CATALOG_THEMES_DIR,
      true,
    )
    const projectThemes = themeFilesFromList(
      proj.files ?? [],
      state.themesDir,
      false,
    )
    ctx.setState({ ...ctx.getState(), catalogThemes, projectThemes })
    state = ctx.getState()
    const prefer =
      opts?.themeId ||
      state.themeId ||
      projectThemes[0]?.id ||
      catalogThemes.find((t) => t.id === 'studio')?.id ||
      projectThemes.find((t) => t.id === 'studio')?.id ||
      catalogThemes.find((t) => t.id === 'ship')?.id ||
      catalogThemes[0]?.id ||
      'studio'
    if (state.dirty && state.themeDoc && !opts?.themeId) {
      ctx.setState({ ...ctx.getState(), loading: false })
      ctx.emit()
      return
    }
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
      content,
    })
    ctx.setState({
      ...ctx.getState(),
      forkedFrom: srcId,
      themePath: dest_path,
      themeId: id,
      lastWriteError: null,
    })
    await loadThemesOn(ctx, { themeId: id })
    return true
  } catch (e) {
    ctx.setState({
      ...ctx.getState(),
      lastWriteError: userError(e, 'Could not create that theme — try again.'),
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

export function emptyDesignState(): DesignSurfaceState {
  return {
    projectRoot: '',
    themesDir: 'apps/studio/tokens/themes',
    groupId: DEFAULT_GROUP_ID,
    density: 'default',
    themeId: 'ship',
    modeId: 'dark',
    forkedFrom: null,
    themePath: null,
    themeDoc: null,
    dirty: false,
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
    diffRows: [],
    statusNote: 'Pick a token group to edit',
  }
}
