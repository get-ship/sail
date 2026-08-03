
export type DensityId = 'compact' | 'default' | 'spacious'

export const DENSITIES: DensityId[] = ['compact', 'default', 'spacious']

/** Ordered radius scale steps (excludes full sentinel). */
export const RADIUS_ORDERED = ['sm', 'md', 'lg', 'xl'] as const
export type RadiusStep = (typeof RADIUS_ORDERED)[number]

/** Icon size-scale steps used by design surface family editor (subset of full ladder). */
export const ICON_ORDERED = ['sm', 'md', 'lg', 'xl'] as const

export type ScaleStep = {
  key: string
  valuePx: number
  raw: string
}

export type ScaleFamily = {
  /** DTCG group path, e.g. `radius`. */
  path: string
  steps: ScaleStep[]
  ordered: string[]
  scaleTogether: boolean
}

export type InheritNode = {
  id: string
  label: string
  /** Display value when override, else null (inherits). */
  value: string | null
  isOverride: boolean
  /** DTCG path for write (component.button.radius …). */
  path: string
  canReset: boolean
}

export type TokenSite = {
  file: string
  span?: { line?: number; col?: number; start?: number; end?: number }
}

export type BlastRadius = {
  token: string
  components: number
  componentIds: string[]
  sites: TokenSite[]
  label: string
}

export type ThemeFile = {
  /** Basename e.g. ship.json */
  file: string
  /** Theme id (stem or $extensions.ship.themeId). */
  id: string
  /** Workspace-relative path. */
  path: string
  catalog: boolean
}

export type FontRole = 'display' | 'body' | 'mono'

export type TypeRoles = {
  display: string
  body: string
  mono: string
}

export type IconPackState = {
  pack: string
  /** Glyph names available for browse (lucide registry / build manifest). */
  glyphs: string[]
  query: string
}

export type DesignDiffRow = {
  path: string
  from: string
  to: string
}

/** Five front-door design decisions (logical group ids). */
export type DecisionId = 'color' | 'typography' | 'density' | 'radius' | 'icon'

export const DECISION_IDS: DecisionId[] = [
  'color',
  'typography',
  'density',
  'radius',
  'icon',
]

export type DesignSurfaceState = {
  projectRoot: string
  /** apps/<app>/tokens/themes */
  themesDir: string
  /** Active token group (color / space / radius / …). */
  groupId: string
  /** Active front-door decision (subset of groups). */
  decisionId: DecisionId
  density: DensityId
  /** Active working theme id (forked or catalog). */
  themeId: string
  /** Render mode for specimen repaints — follows the shell's theme mode. */
  modeId: 'light' | 'dark'
  /** Catalog origin when forked (e.g. ship). */
  forkedFrom: string | null
  /** Workspace-relative project theme path when writing. */
  themePath: string | null
  /** Parsed theme JSON (project or working copy). */
  themeDoc: Record<string, unknown> | null
  /** Dirty working doc vs last saved. */
  dirty: boolean
  /** Save review open — Used by / usage summary shown before commit. */
  saveReviewOpen: boolean
  catalogThemes: ThemeFile[]
  projectThemes: ThemeFile[]
  radius: ScaleFamily
  inheritance: InheritNode[]
  typeRoles: TypeRoles
  icons: IconPackState
  /** Icon size-scale family (sm/md/lg/xl). */
  iconScale: ScaleFamily
  blast: BlastRadius | null
  inspectOpen: boolean
  overridesOnly: boolean
  tokenSearch: string
  overrideCount: number
  /** Specimen SailRender result (base64 png without data: prefix ok). */
  specimenPng: string | null
  specimenMs: number | null
  loading: boolean
  error: string | null
  lastWriteError: string | null
  /** Plain-language clamp from scale family writes. */
  lastClampNote: string | null
  /** Exportable DTCG diff rows (path → from/to). */
  diffRows: DesignDiffRow[]
  statusNote: string
  /** Theme doc undo stack non-empty. */
  canUndo: boolean
  /** Theme doc redo stack non-empty. */
  canRedo: boolean
}

export type DesignOpsClient = {
  sailThemeList: (args: { path: string }) => Promise<{
    files?: string[]
    ok?: boolean
    path?: string
  }>
  sailThemeRead: (args: { path: string }) => Promise<{
    content: string
    ok?: boolean
    path?: string
  }>
  sailThemeWrite: (args: {
    path: string
    content: string
  }) => Promise<{ ok?: boolean; path?: string; bytes?: number }>
  sailThemeDelete?: (args: {
    path: string
  }) => Promise<{ ok?: boolean; path?: string }>
  sailThemeFork: (args: {
    source_path: string
    dest_path: string
    new_id?: string
    content?: string
  }) => Promise<{ ok?: boolean; dest_path?: string; path?: string }>
  /**
   * SailTheme/Generate — regenerate registered app theme tables (mode=app).
   */
  sailThemeGenerate?: (args: {
    mode?: string
    project?: string
    workspace?: string
    model?: string
    out?: string
  }) => Promise<Record<string, unknown>>
  sailThemeGraph?: (args: { themeId?: string }) => Promise<Record<string, unknown>>
  sailThemeEffectiveIds?: (args: {
    project?: string
  }) => Promise<{ all?: string[]; project?: string[]; catalog?: string[] }>
  sailInspectTokenUsage?: (args: { token: string }) => Promise<{
    token?: string
    components?: number
    componentIds?: string[]
    sites?: unknown[]
  }>
  sailRenderScreenshot?: (args: {
    path?: string
    source?: string
    theme?: string
    mode?: string
    viewport?: { width: number; height: number } | string
  }) => Promise<{
    imagePng?: string
    ms?: number
    theme?: string
  }>
  /** SailTheme/FontAdd — vendor woff2 + tokens. */
  sailThemeFontAdd?: (args: {
    family: string
    axes?: string
  }) => Promise<Record<string, unknown>>
  /** SailTheme/FontSearch — full Google Fonts catalog (bundled + filter). */
  sailThemeFontSearch?: (args: {
    query?: string
    category?: string
    limit?: number
  }) => Promise<{
    families?: string[]
    hits?: Array<{
      family?: string
      category?: string
      popularity?: number
      mono?: boolean
    }>
    count?: number
    catalogSize?: number
    source?: string
  }>
}

export const CATALOG_THEMES_DIR = 'packages/sail/tokens/model/themes'
