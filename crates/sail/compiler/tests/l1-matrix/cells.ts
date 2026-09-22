/**
 * Catalog → Cell[] SSOT for the L1 matrix.
 *
 * Fan-out is not cartesian:
 *   1. rest default × every THEME_META id × that theme's modes
 *   2. axis sweep on sail light + sail dark (other axes at default)
 *   3. interacting pairs only when PAIR_TUNES names them
 *   4. overlays are opened (overlay: true) — never a permanent skip
 *
 * Incomplete catalog coverage (an axis value with no cell) is a failed probe.
 */
import { COMPONENTS, TAGS, type L1Tag } from '../../../../../packages/sail/l1/src/index.ts'
import { THEME_META } from '../../../../../packages/sail/tokens/src/generated/themes-meta.ts'

export const CANONICAL_THEME = 'sail'
export const CANONICAL_MODES = ['light', 'dark'] as const

/** Hosts whose painted surface is off-page until opened. */
export const OVERLAY_HOSTS = new Set([
  'dialog',
  'drawer',
  'popover',
  'tooltip',
  'dropdown',
  'context-menu',
  'hover-card',
  'combobox',
])

/**
 * Known interacting pairs. Empty until a tune miss or ledger row names one.
 * Do not invent a pair list in advance.
 */
export const PAIR_TUNES: readonly { tag: string; axes: readonly [string, string] }[] = []

export type ThemeMode = { theme: string; mode: string }

export type CellKind = 'rest' | 'axis' | 'pair'

export type Contract = {
  role: string
  name?: string
  states?: string[]
}

export type Cell = {
  id: string
  tag: string
  component: string
  theme: string
  mode: string
  kind: CellKind
  axis?: string
  value?: string
  /** Second axis when kind === 'pair'. */
  axisB?: string
  valueB?: string
  props: Record<string, string>
  route: string
  specimen: string
  contract: Contract
  overlay: boolean
  trigger?: Contract
  /** Open surface (listbox / menu / dialog). Distinct from the closed field. */
  surface?: Contract
}

export function themeModes(): ThemeMode[] {
  const out: ThemeMode[] = []
  for (const t of THEME_META) {
    const modes = t.modes.length ? [...t.modes] : [t.defaultMode ?? 'light']
    for (const mode of modes) out.push({ theme: t.id, mode })
  }
  return out
}

export function canonicalModes(): ThemeMode[] {
  return CANONICAL_MODES.map((mode) => ({ theme: CANONICAL_THEME, mode }))
}

export function hostTags(): L1Tag[] {
  const hosts = new Set(COMPONENTS.map((c) => c.host))
  return TAGS.filter((t) => t.role === 'host' && hosts.has(t.tag))
}

export function tagOf(name: string): L1Tag {
  const t = TAGS.find((x) => x.tag === name)
  if (!t) throw new Error(`cells: unknown tag ${name}`)
  return t
}

export function componentOf(tag: string): string {
  return tagOf(tag).component
}

export function routeOf(tag: string): string {
  return `/${componentOf(tag)}`
}

export function isBoolAxis(tag: L1Tag, axis: string): boolean {
  const a = tag.axes.find((x) => x.name === axis)
  if (!a) return false
  return a.values.length === 2 && a.values.includes('true') && a.values.includes('false')
}

export function defaultProps(tag: L1Tag): Record<string, string> {
  const out: Record<string, string> = {}
  for (const p of tag.props) {
    if (p.default !== undefined) out[p.name] = p.default
  }
  return out
}

export function specimenName(
  tag: string,
  kind: CellKind,
  axis?: string,
  value?: string,
  axisB?: string,
  valueB?: string,
): string {
  if (kind === 'pair' && axis && axisB) return `${tag} ${axis} ${value} ${axisB} ${valueB}`
  if (kind === 'axis' && axis) return `${tag} ${axis} ${value}`
  return `${tag} rest`
}

/**
 * Published contract used by getByContract.
 * Unique specimen names live on the fixture (children / label).
 * Hosts whose product name is fixed (spinner Loading) keep that name —
 * the probe scopes to the specimen heading.
 */
export function contractFor(
  tag: string,
  specimen: string,
  props: Record<string, string> = {},
): Contract {
  switch (tag) {
    case 'a':
      return { role: 'link', name: specimen }
    case 'accordion':
      return { role: 'group', name: specimen }
    case 'alert':
      return {
        role: props.tone === 'warning' || props.tone === 'danger' ? 'alert' : 'status',
        name: specimen,
      }
    case 'avatar':
      return { role: 'img', name: specimen }
    case 'badge':
      return { role: 'status', name: specimen }
    case 'breadcrumb':
      return { role: 'navigation', name: 'Breadcrumb' }
    case 'button':
      return { role: 'button', name: specimen }
    case 'card':
      return { role: 'region', name: 'Card' }
    case 'checkbox':
      return { role: 'checkbox', name: specimen }
    case 'chip':
      return { role: 'button', name: specimen }
    case 'code-block':
      return { role: 'region', name: 'Code' }
    case 'color':
      return { role: 'textbox', name: specimen }
    case 'combobox':
      return { role: 'combobox', name: specimen }
    case 'context-menu':
      return { role: 'menu', name: specimen }
    case 'date':
      return { role: 'textbox', name: specimen }
    case 'dialog':
      return { role: 'dialog', name: specimen }
    case 'drawer':
      return { role: 'dialog', name: specimen }
    case 'dropdown':
      return { role: 'menu', name: specimen }
    case 'empty-state':
      return { role: props.variant === 'connecting' ? 'status' : 'note', name: specimen }
    case 'error-boundary':
      return { role: 'group', name: specimen }
    case 'error-state':
      return { role: 'alert', name: specimen }
    case 'field':
      return { role: 'group', name: specimen }
    case 'fieldset':
      return { role: 'group', name: specimen }
    case 'form':
      return { role: 'form', name: specimen }
    case 'list-row':
      return { role: 'listitem', name: specimen }
    case 'file':
      return { role: 'group', name: specimen }
    case 'hover-card':
      return { role: 'dialog', name: specimen }
    case 'icon':
      return { role: 'img', name: specimen }
    case 'input':
      return {
        role: inputRole(props.type),
        name: specimen,
        states: props.type === 'hidden' ? ['hidden'] : undefined,
      }
    case 'kbd':
      return { role: 'group' }
    case 'label':
      // Native <label> has no role; its contract is the named control.
      return { role: 'textbox', name: specimen }
    case 'legend':
      return { role: 'group', name: specimen }
    case 'menu-item':
      return { role: 'button', name: specimen }
    case 'meter':
      return { role: 'meter', name: specimen }
    case 'navbar':
      return { role: 'navigation', name: 'Application' }
    case 'page':
      return { role: 'region', name: 'Page' }
    case 'page-item':
      return { role: 'button', name: specimen }
    case 'pagination':
      return { role: 'navigation', name: 'Pagination' }
    case 'pane':
      return { role: 'group', name: 'Pane' }
    case 'popover':
      return { role: 'dialog', name: specimen }
    case 'progress':
      return { role: 'progressbar', name: specimen }
    case 'prose':
      return { role: 'article', name: specimen }
    case 'radio':
      return { role: 'radio', name: specimen }
    case 'radio-group':
      return { role: 'radiogroup', name: specimen }
    case 'resizable':
      return { role: 'separator', name: 'Resize' }
    case 'scroll-area':
      return { role: 'region', name: 'Scroll' }
    case 'select':
      return {
        role: props.multiple === 'true' ? 'listbox' : 'combobox',
        name: specimen,
      }
    case 'separator':
      return { role: 'separator', name: specimen }
    case 'skeleton':
      return { role: 'status', name: 'Loading' }
    case 'slider':
      return { role: 'slider', name: specimen }
    case 'spinner':
      return { role: 'status', name: 'Loading' }
    case 'switch':
      return { role: 'switch', name: specimen }
    case 'tab':
      return { role: 'tab', name: specimen }
    case 'tab-list':
      return { role: 'tablist', name: specimen }
    case 'tab-panel':
      return { role: 'tabpanel', name: specimen }
    case 'table':
      return { role: 'table' }
    case 'tabs':
      return { role: 'tab', name: specimen }
    case 'td':
      return { role: 'cell', name: specimen }
    case 'textarea':
      return { role: 'textbox', name: specimen }
    case 'th':
      return { role: 'columnheader', name: specimen }
    case 'theme-picker':
      if (props.inline === 'true' || props.modeOnly === 'true') {
        return { role: 'button', name: 'Color mode' }
      }
      return { role: 'button' }
    case 'ticker':
      return { role: 'status', name: specimen }
    case 'toast':
      return { role: 'status', name: specimen }
    case 'toast-region':
      return { role: 'region', name: 'Notifications' }
    case 'toggle':
      return { role: 'button', name: specimen }
    case 'toggle-group':
      return { role: 'button', name: 'Day' }
    case 'toolbar':
      return { role: 'toolbar', name: 'Toolbar' }
    case 'tooltip':
      return { role: 'tooltip', name: specimen }
    case 'caption':
      return { role: 'caption', name: specimen }
    default:
      throw new Error(`cells: no contract for tag ${tag}`)
  }
}

function inputRole(type: string | undefined): string {
  if (type === 'search') return 'searchbox'
  if (type === 'number') return 'spinbutton'
  return 'textbox'
}

export function overlayTrigger(tag: string, specimen: string): Contract | undefined {
  if (!OVERLAY_HOSTS.has(tag)) return undefined
  if (tag === 'combobox') return { role: 'combobox', name: specimen }
  if (tag === 'tooltip') return { role: 'button', name: specimen }
  if (tag === 'context-menu') return { role: 'button', name: `${specimen} target` }
  return { role: 'button', name: `Open ${specimen}` }
}

/** Open overlay surface — not the closed trigger / field. */
export function overlaySurface(tag: string, specimen: string): Contract | undefined {
  if (!OVERLAY_HOSTS.has(tag)) return undefined
  if (tag === 'combobox') return { role: 'listbox' }
  if (tag === 'tooltip') return { role: 'tooltip', name: specimen }
  if (tag === 'dropdown' || tag === 'context-menu') return { role: 'menu', name: specimen }
  if (tag === 'hover-card') return { role: 'dialog', name: specimen }
  return contractFor(tag, specimen)
}

const SELECTED_ROLES = new Set([
  'tab',
  'option',
  'row',
  'gridcell',
  'columnheader',
  'rowheader',
  'treeitem',
])

function withStates(entry: Contract, props: Record<string, string>): Contract {
  const states = [...(entry.states ?? [])]
  if (props.disabled === 'true' && !states.includes('disabled')) states.push('disabled')
  if (props.checked === 'true' && !states.includes('checked')) states.push('checked')
  // Playwright getByRole only accepts `selected` on a closed role set — not button/chip.
  if (props.selected === 'true' && SELECTED_ROLES.has(entry.role) && !states.includes('selected')) {
    states.push('selected')
  }
  if (states.length === 0) return entry
  return { ...entry, states }
}

function cellId(
  c: Omit<Cell, 'id' | 'contract' | 'route' | 'specimen' | 'overlay' | 'trigger' | 'surface'>,
): string {
  const axis = c.kind === 'pair' ? `${c.axis}=${c.value}+${c.axisB}=${c.valueB}` : c.axis ? `${c.axis}=${c.value}` : '-'
  return `${c.tag}::${c.kind}::${axis}::${c.theme}/${c.mode}`
}

function finish(
  partial: Omit<Cell, 'id' | 'contract' | 'route' | 'specimen' | 'overlay' | 'trigger' | 'surface'>,
): Cell {
  const specimen = specimenName(
    partial.tag,
    partial.kind,
    partial.axis,
    partial.value,
    partial.axisB,
    partial.valueB,
  )
  const overlay = OVERLAY_HOSTS.has(partial.tag)
  return {
    ...partial,
    id: cellId(partial),
    route: routeOf(partial.tag),
    specimen,
    contract: withStates(contractFor(partial.tag, specimen, partial.props), partial.props),
    overlay,
    trigger: overlayTrigger(partial.tag, specimen),
    surface: overlaySurface(partial.tag, specimen),
  }
}

function sweepProps(tag: L1Tag, axis: string, value: string): Record<string, string> {
  const props = defaultProps(tag)
  props[axis] = value
  return props
}

export function cells(): Cell[] {
  const out: Cell[] = []
  const themes = themeModes()
  const canon = canonicalModes()

  for (const host of hostTags()) {
    for (const tm of themes) {
      out.push(
        finish({
          tag: host.tag,
          component: host.component,
          theme: tm.theme,
          mode: tm.mode,
          kind: 'rest',
          props: defaultProps(host),
        }),
      )
    }
  }

  for (const tag of TAGS) {
    for (const axis of tag.axes) {
      for (const value of axis.values) {
        for (const tm of canon) {
          out.push(
            finish({
              tag: tag.tag,
              component: tag.component,
              theme: tm.theme,
              mode: tm.mode,
              kind: 'axis',
              axis: axis.name,
              value,
              props: sweepProps(tag, axis.name, value),
            }),
          )
        }
      }
    }
  }

  for (const pair of PAIR_TUNES) {
    const tag = tagOf(pair.tag)
    const a = tag.axes.find((x) => x.name === pair.axes[0])
    const b = tag.axes.find((x) => x.name === pair.axes[1])
    if (!a || !b) continue
    for (const va of a.values) {
      for (const vb of b.values) {
        const props = defaultProps(tag)
        props[a.name] = va
        props[b.name] = vb
        for (const tm of canon) {
          out.push(
            finish({
              tag: tag.tag,
              component: tag.component,
              theme: tm.theme,
              mode: tm.mode,
              kind: 'pair',
              axis: a.name,
              value: va,
              axisB: b.name,
              valueB: vb,
              props,
            }),
          )
        }
      }
    }
  }

  return out
}

/** Axis values in TAGS that have no generated cell. Empty = complete fan-out. */
export function missingAxisCoverage(list: Cell[] = cells()): string[] {
  const seen = new Set<string>()
  for (const c of list) {
    if (c.axis && c.value !== undefined) seen.add(`${c.tag}::${c.axis}::${c.value}`)
    if (c.axisB && c.valueB !== undefined) seen.add(`${c.tag}::${c.axisB}::${c.valueB}`)
  }
  const missing: string[] = []
  for (const tag of TAGS) {
    for (const axis of tag.axes) {
      for (const value of axis.values) {
        const key = `${tag.tag}::${axis.name}::${value}`
        if (!seen.has(key)) missing.push(key)
      }
    }
  }
  return missing
}

export function hostRoutes(): string[] {
  return COMPONENTS.map((c) => `/${c.host}`)
}
