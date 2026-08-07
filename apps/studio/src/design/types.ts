import type { StudioOpsMethods } from '@ship/sail-studio-client'

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
  /**
   * Mode names from DTCG `modes` (project themes). Catalog rows usually omit
   * this — paint uses package `themes-meta` instead. Hydrated at list load so
   * project swatches keep correct light/dark tables without package contamination.
   */
  modes?: ('light' | 'dark')[]
  /** From `$extensions.ship.defaultMode` when present on the project theme JSON. */
  defaultMode?: 'light' | 'dark'
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
  /** apps/<app>/themes (modern; legacy tokens/themes dual-read in compiler/ops) */
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

/** Themes/design surface client — derived from generated ops (no hand-written shapes). */
export type DesignOpsClient = Pick<
  StudioOpsMethods,
  'sailThemeList' | 'sailThemeRead' | 'sailThemeWrite' | 'sailThemeFork'
> &
  Partial<
    Pick<
      StudioOpsMethods,
      | 'sailThemeDelete'
      | 'sailThemeGenerate'
      | 'sailThemeGraph'
      | 'sailThemeEffectiveIds'
      | 'sailInspectTokenUsage'
      | 'sailRenderScreenshot'
      | 'sailThemeFontAdd'
      | 'sailThemeFontSearch'
    >
  >

export const CATALOG_THEMES_DIR = 'packages/sail/tokens/model/themes'
