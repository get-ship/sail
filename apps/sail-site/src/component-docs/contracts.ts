/**
 * Index of surface contracts for first-class (and contracted) docs packages.
 * Checker + tooling import from here.
 */
import { contract as alert } from './alert/contract'
import { contract as badge } from './badge/contract'
import { contract as button } from './button/contract'
import { contract as chip } from './chip/contract'
import { contract as field } from './field/contract'
import { contract as fieldset } from './fieldset/contract'
import { contract as icon } from './icon/contract'
import { contract as label } from './label/contract'
import { contract as meter } from './meter/contract'
import { contract as progress } from './progress/contract'
import { contract as radioGroup } from './radio-group/contract'
import { contract as separator } from './separator/contract'
import { contract as skeleton } from './skeleton/contract'
import { contract as slider } from './slider/contract'
import { contract as spinner } from './spinner/contract'
import { contract as switchToggle } from './switch/contract'
import { contract as input } from './input/contract'
import { contract as textarea } from './textarea/contract'
import { contract as select } from './select/contract'
import { contract as a } from './a/contract'
import { contract as kbd } from './kbd/contract'
import { contract as prose } from './prose/contract'
import { contract as codeBlock } from './code-block/contract'
import { contract as emptyState } from './empty-state/contract'
import { contract as errorState } from './error-state/contract'
import { contract as errorBoundary } from './error-boundary/contract'
import { contract as scrollArea } from './scroll-area/contract'
import { contract as ticker } from './ticker/contract'
import { contract as table } from './table/contract'
import { contract as pagination } from './pagination/contract'
import { contract as toggleGroup } from './toggle-group/contract'
import { contract as tabs } from './tabs/contract'
import { contract as navbar } from './navbar/contract'
import { contract as card } from './card/contract'
import { contract as avatar } from './avatar/contract'
import { contract as breadcrumb } from './breadcrumb/contract'
import { contract as dialog } from './dialog/contract'
import { contract as drawer } from './drawer/contract'
import { contract as popover } from './popover/contract'
import { contract as tooltip } from './tooltip/contract'
import { contract as dropdown } from './dropdown/contract'
import { contract as hoverCard } from './hover-card/contract'
import { contract as contextMenu } from './context-menu/contract'
import { contract as toast } from './toast/contract'
import { contract as accordion } from './accordion/contract'
import { contract as combobox } from './combobox/contract'
import { contract as themePicker } from './theme-picker/contract'
import { contract as checkbox } from './checkbox/contract'
import { contract as color } from './color/contract'
import { contract as date } from './date/contract'
import { contract as file } from './file/contract'
import { contract as resizable } from './resizable/contract'
import type { SurfaceContract } from './contract-types'

export const CONTRACTS: readonly SurfaceContract[] = [
  button,
  badge,
  fieldset,
  chip,
  alert,
  separator,
  spinner,
  skeleton,
  icon,
  progress,
  meter,
  label,
  field,
  radioGroup,
  switchToggle,
  slider,
  input,
  textarea,
  select,
  a,
  kbd,
  prose,
  codeBlock,
  emptyState,
  errorState,
  errorBoundary,
  scrollArea,
  ticker,
  table,
  pagination,
  toggleGroup,
  tabs,
  navbar,
  card,
  avatar,
  breadcrumb,
  dialog,
  drawer,
  popover,
  tooltip,
  dropdown,
  hoverCard,
  contextMenu,
  toast,
  accordion,
  combobox,
  themePicker,
  checkbox,
  color,
  date,
  file,
  resizable,
]

export const CONTRACT_BY_SLUG: Record<string, SurfaceContract> = Object.fromEntries(
  CONTRACTS.map((c) => [c.slug, c]),
)

export function contractFor(slug: string): SurfaceContract | null {
  return CONTRACT_BY_SLUG[slug] ?? null
}
