/**
 * Components page domain types — library records + inspect / detail VMs.
 */

export type ComponentOrigin = 'project' | 'pack'

export type ComponentLayer = 'app' | 'L2' | 'unknown'

export type ComponentProp = {
  name: string
  type: string
  defaultValue: string
  required: boolean
  description: string
}

export type ComponentRecord = {
  id: string
  displayName: string
  origin: ComponentOrigin
  package: string
  path: string
  layer: ComponentLayer
  description: string
  props: ComponentProp[]
  slots: string[]
  issueCount: number
  ssrTierSummary: string
  hasPreview: boolean
  threadCount: number
  /** True when Open in Edit has a real project path. */
  canOpenInEdit: boolean
}

export type ComponentRowUi = {
  id: string
  title: string
  meta: string
  path: string
  origin: ComponentOrigin
  package: string
  selected: boolean
  originLabel: string
  originTone: string
  issueLabel: string
  hasIssues: boolean
  key: string
}

export type ComponentMode = 'library' | 'inspect'

export type DetailTabId = 'overview' | 'contract' | 'health' | 'theme'

export type PropRowUi = {
  name: string
  type: string
  defaultValue: string
  requiredLabel: string
  description: string
  key: string
}

export type KvRowUi = {
  label: string
  value: string
  key: string
}

export type SurfaceOption = {
  value: string
  label: string
  key: string
  /** route | file | open — for grouping in inspect list */
  kind?: 'route' | 'file' | 'open'
  selected?: boolean
}

export type InspectNodeDetail = {
  nodeId: string
  tag: string
  layer: string
  path: string
  spanLabel: string
  line: number
  col: number
}

export type IssueRowUi = {
  id: string
  severity: string
  tone: string
  message: string
  spanLabel: string
  key: string
}

export type LayoutBoxUi = {
  id: string
  tag: string
  dims: string
  flags: string
  key: string
}

export type StateRowUi = {
  id: string
  name: string
  kind: string
  tierLabel: string
  tierTone: string
  key: string
}

/** CEM-ish catalog shape (components.gen `as const` + fixtures). */
export type CatalogAttribute = {
  readonly name?: string
  readonly type?: { readonly text?: string } | string
  readonly default?: string
  readonly required?: boolean
  readonly description?: string
}

export type CatalogDeclaration = {
  readonly kind?: string
  readonly name?: string
  readonly tagName?: string
  readonly customElement?: boolean
  readonly description?: string
  readonly attributes?: readonly CatalogAttribute[]
  readonly slots?: readonly (string | { readonly name?: string })[]
  readonly cssParts?: unknown
  readonly cssProperties?: unknown
}

export type CatalogModule = {
  readonly kind?: string
  readonly path?: string
  readonly package?: string
  readonly declarations?: readonly CatalogDeclaration[]
}

export type ComponentsCatalog = {
  readonly schemaVersion?: string
  readonly modules?: readonly CatalogModule[]
}
