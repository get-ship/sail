// Static import map of component-docs/*/meta.ts.
// Hub tile exists iff a package meta is listed here (and the meta.ts file exists).
import { meta as buttonMeta } from './button/meta'
import { meta as aMeta } from './a/meta'
import { meta as inputMeta } from './input/meta'
import { meta as textareaMeta } from './textarea/meta'
import { meta as selectMeta } from './select/meta'
import { meta as comboboxMeta } from './combobox/meta'
import { meta as checkboxMeta } from './checkbox/meta'
import { meta as dateMeta } from './date/meta'
import { meta as fileMeta } from './file/meta'
import { meta as colorMeta } from './color/meta'
import { meta as radioMeta } from './radio/meta'
import { meta as radioGroupMeta } from './radio-group/meta'
import { meta as switchToggleMeta } from './switch/meta'
import { meta as sliderMeta } from './slider/meta'
import { meta as toggleGroupMeta } from './toggle-group/meta'
import { meta as fieldMeta } from './field/meta'
import { meta as labelMeta } from './label/meta'
import { meta as fieldsetMeta } from './fieldset/meta'
import { meta as dropzoneMeta } from './dropzone/meta'
import { meta as themePickerMeta } from './theme-picker/meta'
import { meta as alertMeta } from './alert/meta'
import { meta as progressBarMeta } from './progress/meta'
import { meta as meterMeta } from './meter/meta'
import { meta as spinnerMeta } from './spinner/meta'
import { meta as skeletonMeta } from './skeleton/meta'
import { meta as emptyStateMeta } from './empty-state/meta'
import { meta as errorStateMeta } from './error-state/meta'
import { meta as errorBoundaryMeta } from './error-boundary/meta'
import { meta as toastMeta } from './toast/meta'
import { meta as badgeMeta } from './badge/meta'
import { meta as chipMeta } from './chip/meta'
import { meta as cardMeta } from './card/meta'
import { meta as avatarMeta } from './avatar/meta'
import { meta as iconMeta } from './icon/meta'
import { meta as separatorMeta } from './separator/meta'
import { meta as kbdMeta } from './kbd/meta'
import { meta as codeBlockMeta } from './code-block/meta'
import { meta as accordionMeta } from './accordion/meta'
import { meta as proseMeta } from './prose/meta'
import { meta as tooltipMeta } from './tooltip/meta'
import { meta as hoverCardMeta } from './hover-card/meta'
import { meta as popoverMeta } from './popover/meta'
import { meta as dropdownMeta } from './dropdown/meta'
import { meta as contextMenuMeta } from './context-menu/meta'
import { meta as dialogMeta } from './dialog/meta'
import { meta as tabsMeta } from './tabs/meta'
import { meta as breadcrumbMeta } from './breadcrumb/meta'
import { meta as paginationMeta } from './pagination/meta'
import { meta as navbarMeta } from './navbar/meta'
import { meta as tableMeta } from './table/meta'
import { meta as layoutMeta } from './layout/meta'
import { meta as scrollAreaMeta } from './scroll-area/meta'
import { meta as tickerMeta } from './ticker/meta'
import type { DocMeta } from './types'

export const DOC_PACKAGES: readonly DocMeta[] = [
  buttonMeta,
  aMeta,
  inputMeta,
  textareaMeta,
  selectMeta,
  comboboxMeta,
  checkboxMeta,
  dateMeta,
  fileMeta,
  colorMeta,
  radioMeta,
  radioGroupMeta,
  switchToggleMeta,
  sliderMeta,
  toggleGroupMeta,
  fieldMeta,
  labelMeta,
  fieldsetMeta,
  dropzoneMeta,
  themePickerMeta,
  alertMeta,
  progressBarMeta,
  meterMeta,
  spinnerMeta,
  skeletonMeta,
  emptyStateMeta,
  errorStateMeta,
  errorBoundaryMeta,
  toastMeta,
  badgeMeta,
  chipMeta,
  cardMeta,
  avatarMeta,
  iconMeta,
  separatorMeta,
  kbdMeta,
  codeBlockMeta,
  accordionMeta,
  proseMeta,
  tooltipMeta,
  hoverCardMeta,
  popoverMeta,
  dropdownMeta,
  contextMenuMeta,
  dialogMeta,
  tabsMeta,
  breadcrumbMeta,
  paginationMeta,
  navbarMeta,
  tableMeta,
  layoutMeta,
  scrollAreaMeta,
  tickerMeta,
]

export const DOC_BY_SLUG: Record<string, DocMeta> = Object.fromEntries(
  DOC_PACKAGES.map((m) => [m.slug, m]),
)

export function docMeta(slug: string): DocMeta | null {
  return DOC_BY_SLUG[slug] ?? null
}

/** First-class doc hosts (every package; missing CE → empty state). */
export const FIRST_CLASS_HOSTS = new Set(
  DOC_PACKAGES.map((m) => m.slug),
)
