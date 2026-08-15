<!--
  /components/[tag] — thin detail shell.
  Docs SSOT: apps/sail-site/src/component-docs/<slug>/meta.ts
  Mounts exactly one docs host:
    - first-class package CE (doc-button, doc-badge, …) when meta.host is doc-*
    - explicit missing-host empty state if first-class meta but no CE wired
    - empty state when no package / pack meta
-->

<scope>
  style page = { size({ w: full, max-w: breakpoint-desktop }).margin({ x: auto }) }
  style emptyEnter = { animate: { enter: play(slide-up) } }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { docMeta, FIRST_CLASS_HOSTS } from '../../../src/compositions/component-docs-registry'
import { detailFor, type DetailEntry } from '../../../src/compositions/component-details'
import {
  PACK_PROPS,
  PACK_TAGS,
  packBlurbFor,
  packLabelFor,
} from '../../../src/compositions/component-pack-meta'

/** Slugs with a compiled doc-<slug> CE mounted below. Keep in sync when peeling. */
const DOC_CE_SLUGS = new Set([
  'button',
  'badge',
  'fieldset',
  'chip',
  'alert',
  'separator',
  'spinner',
  'skeleton',
  'icon',
  'progress',
  'meter',
  'label',
  'field',
  'radio-group',
  'switch',
  'slider',
  'input',
  'checkbox',
  'date',
  'file',
  'color',
  'textarea',
  'select',
  'a',
  'kbd',
  'prose',
  'code-block',
  'empty-state',
  'error-state',
  'error-boundary',
  'scroll-area',
  'ticker',
  'table',
  'pagination',
  'toggle-group',
  'tabs',
  'navbar',
  'card',
  'avatar',
  'breadcrumb',
  'dialog',
  'drawer',
  'popover',
  'tooltip',
  'dropdown',
  'hover-card',
  'context-menu',
  'toast',
  'accordion',
  'combobox',
  'theme-picker',
  'resizable',
])

export default controller((ctx: ControllerScope<Scope>) => {
  // radio is a radio-group part — leftover /components/radio folds onto the host.
  const tag = derived(() => {
    const raw = ctx.route.params.tag?.() ?? ''
    return raw === 'radio' ? 'radio-group' : raw
  })
  const meta = derived(() => docMeta(tag()))
  const entry = derived<DetailEntry | null>(() => detailFor(tag()) ?? null)
  const knownDoc = derived(() => meta() !== null)
  const knownPack = derived(() => PACK_TAGS.has(tag()))
  const showViewer = derived(() => knownDoc() || knownPack())

  // FIRST_CLASS_HOSTS is SSOT — every docs package slug.
  const firstClass = derived(() => FIRST_CLASS_HOSTS.has(tag()))
  const hasDocCe = derived(() => DOC_CE_SLUGS.has(tag()))
  const useDocHost = derived(() => firstClass() && hasDocCe())
  const missingDocHost = derived(() => showViewer() && !useDocHost())

  const useButton = derived(() => tag() === 'button' && useDocHost())
  const useBadge = derived(() => tag() === 'badge' && useDocHost())
  const useFieldset = derived(() => tag() === 'fieldset' && useDocHost())
  const useChip = derived(() => tag() === 'chip' && useDocHost())
  const useAlert = derived(() => tag() === 'alert' && useDocHost())
  const useSeparator = derived(() => tag() === 'separator' && useDocHost())
  const useSpinner = derived(() => tag() === 'spinner' && useDocHost())
  const useSkeleton = derived(() => tag() === 'skeleton' && useDocHost())
  const useIcon = derived(() => tag() === 'icon' && useDocHost())
  const useProgress = derived(() => tag() === 'progress' && useDocHost())
  const useMeter = derived(() => tag() === 'meter' && useDocHost())
  const useLabel = derived(() => tag() === 'label' && useDocHost())
  const useField = derived(() => tag() === 'field' && useDocHost())
  const useRadioGroup = derived(() => tag() === 'radio-group' && useDocHost())
  const useSwitch = derived(() => tag() === 'switch' && useDocHost())
  const useSlider = derived(() => tag() === 'slider' && useDocHost())
  const useInput = derived(() => tag() === 'input' && useDocHost())
  const useCheckbox = derived(() => tag() === 'checkbox' && useDocHost())
  const useDate = derived(() => tag() === 'date' && useDocHost())
  const useFile = derived(() => tag() === 'file' && useDocHost())
  const useColor = derived(() => tag() === 'color' && useDocHost())
  const useTextarea = derived(() => tag() === 'textarea' && useDocHost())
  const useSelect = derived(() => tag() === 'select' && useDocHost())
  const useA = derived(() => tag() === 'a' && useDocHost())
  const useKbd = derived(() => tag() === 'kbd' && useDocHost())
  const useProse = derived(() => tag() === 'prose' && useDocHost())
  const useCodeBlock = derived(() => tag() === 'code-block' && useDocHost())
  const useEmptyState = derived(() => tag() === 'empty-state' && useDocHost())
  const useErrorState = derived(() => tag() === 'error-state' && useDocHost())
  const useErrorBoundary = derived(() => tag() === 'error-boundary' && useDocHost())
  const useScrollArea = derived(() => tag() === 'scroll-area' && useDocHost())
  const useTicker = derived(() => tag() === 'ticker' && useDocHost())
  const useTable = derived(() => tag() === 'table' && useDocHost())
  const usePagination = derived(() => tag() === 'pagination' && useDocHost())
  const useToggleGroup = derived(() => tag() === 'toggle-group' && useDocHost())
  const useTabs = derived(() => tag() === 'tabs' && useDocHost())
  const useNavbar = derived(() => tag() === 'navbar' && useDocHost())
  const useCard = derived(() => tag() === 'card' && useDocHost())
  const useAvatar = derived(() => tag() === 'avatar' && useDocHost())
  const useBreadcrumb = derived(() => tag() === 'breadcrumb' && useDocHost())
  const useDialog = derived(() => tag() === 'dialog' && useDocHost())
  const useDrawer = derived(() => tag() === 'drawer' && useDocHost())
  const usePopover = derived(() => tag() === 'popover' && useDocHost())
  const useTooltip = derived(() => tag() === 'tooltip' && useDocHost())
  const useDropdown = derived(() => tag() === 'dropdown' && useDocHost())
  const useHoverCard = derived(() => tag() === 'hover-card' && useDocHost())
  const useContextMenu = derived(() => tag() === 'context-menu' && useDocHost())
  const useToast = derived(() => tag() === 'toast' && useDocHost())
  const useAccordion = derived(() => tag() === 'accordion' && useDocHost())
  const useCombobox = derived(() => tag() === 'combobox' && useDocHost())
  const useThemePicker = derived(() => tag() === 'theme-picker' && useDocHost())
  const useResizable = derived(() => tag() === 'resizable' && useDocHost())

  const heading = () => meta()?.heading ?? entry()?.heading ?? tag()
  const blurb = () => meta()?.blurb ?? entry()?.blurb ?? packBlurbFor(tag())
  const packLabel = () => (knownPack() ? packLabelFor(tag()) : 'Component primitives')
  const packProps = () => PACK_PROPS[tag()] ?? []

  return {
    tag,
    meta,
    entry,
    knownDoc,
    knownPack,
    showViewer,
    firstClass,
    missingDocHost,
    useButton,
    useBadge,
    useFieldset,
    useChip,
    useAlert,
    useSeparator,
    useSpinner,
    useSkeleton,
    useIcon,
    useProgress,
    useMeter,
    useLabel,
    useField,
    useRadioGroup,
    useSwitch,
    useSlider,
    useInput,
    useCheckbox,
    useDate,
    useFile,
    useColor,
    useTextarea,
    useSelect,
    useA,
    useKbd,
    useProse,
    useCodeBlock,
    useEmptyState,
    useErrorState,
    useErrorBoundary,
    useScrollArea,
    useTicker,
    useTable,
    usePagination,
    useToggleGroup,
    useTabs,
    useNavbar,
    useCard,
    useAvatar,
    useBreadcrumb,
    useDialog,
    useDrawer,
    usePopover,
    useTooltip,
    useDropdown,
    useHoverCard,
    useContextMenu,
    useToast,
    useAccordion,
    useCombobox,
    useThemePicker,
    useResizable,

    heading,
    blurb,
    packLabel,
    packProps,
  }
})
</controller>

<column gap="4" style(page)>
  <if {useButton()}>
    <doc-button></doc-button>
  </if>
  <if {useBadge()}>
    <doc-badge></doc-badge>
  </if>
  <if {useFieldset()}>
    <doc-fieldset></doc-fieldset>
  </if>
  <if {useChip()}>
    <doc-chip></doc-chip>
  </if>
  <if {useAlert()}>
    <doc-alert></doc-alert>
  </if>
  <if {useSeparator()}>
    <doc-separator></doc-separator>
  </if>
  <if {useSpinner()}>
    <doc-spinner></doc-spinner>
  </if>
  <if {useSkeleton()}>
    <doc-skeleton></doc-skeleton>
  </if>
  <if {useIcon()}>
    <doc-icon></doc-icon>
  </if>
  <if {useProgress()}>
    <doc-progress></doc-progress>
  </if>
  <if {useMeter()}>
    <doc-meter></doc-meter>
  </if>
  <if {useLabel()}>
    <doc-label></doc-label>
  </if>
  <if {useField()}>
    <doc-field></doc-field>
  </if>
  <if {useRadioGroup()}>
    <doc-radio-group></doc-radio-group>
  </if>
  <if {useSwitch()}>
    <doc-switch></doc-switch>
  </if>
  <if {useSlider()}>
    <doc-slider></doc-slider>
  </if>
  <if {useInput()}>
    <doc-input></doc-input>
  </if>
  <if {useCheckbox()}>
    <doc-checkbox></doc-checkbox>
  </if>
  <if {useDate()}>
    <doc-date></doc-date>
  </if>
  <if {useFile()}>
    <doc-file></doc-file>
  </if>
  <if {useColor()}>
    <doc-color></doc-color>
  </if>
  <if {useTextarea()}>
    <doc-textarea></doc-textarea>
  </if>
  <if {useSelect()}>
    <doc-select></doc-select>
  </if>
  <if {useA()}>
    <doc-a></doc-a>
  </if>
  <if {useKbd()}>
    <doc-kbd></doc-kbd>
  </if>
  <if {useProse()}>
    <doc-prose></doc-prose>
  </if>
  <if {useCodeBlock()}>
    <doc-code-block></doc-code-block>
  </if>
  <if {useEmptyState()}>
    <doc-empty-state></doc-empty-state>
  </if>
  <if {useErrorState()}>
    <doc-error-state></doc-error-state>
  </if>
  <if {useErrorBoundary()}>
    <doc-error-boundary></doc-error-boundary>
  </if>
  <if {useScrollArea()}>
    <doc-scroll-area></doc-scroll-area>
  </if>
  <if {useTicker()}>
    <doc-ticker></doc-ticker>
  </if>
  <if {useTable()}>
    <doc-table></doc-table>
  </if>
  <if {usePagination()}>
    <doc-pagination></doc-pagination>
  </if>
  <if {useToggleGroup()}>
    <doc-toggle-group></doc-toggle-group>
  </if>
  <if {useTabs()}>
    <doc-tabs></doc-tabs>
  </if>
  <if {useNavbar()}>
    <doc-navbar></doc-navbar>
  </if>
  <if {useCard()}>
    <doc-card></doc-card>
  </if>
  <if {useAvatar()}>
    <doc-avatar></doc-avatar>
  </if>
  <if {useBreadcrumb()}>
    <doc-breadcrumb></doc-breadcrumb>
  </if>
  <if {useDialog()}>
    <doc-dialog></doc-dialog>
  </if>
  <if {useDrawer()}>
    <doc-drawer></doc-drawer>
  </if>
  <if {usePopover()}>
    <doc-popover></doc-popover>
  </if>
  <if {useTooltip()}>
    <doc-tooltip></doc-tooltip>
  </if>
  <if {useDropdown()}>
    <doc-dropdown></doc-dropdown>
  </if>
  <if {useHoverCard()}>
    <doc-hover-card></doc-hover-card>
  </if>
  <if {useContextMenu()}>
    <doc-context-menu></doc-context-menu>
  </if>
  <if {useToast()}>
    <doc-toast></doc-toast>
  </if>
  <if {useAccordion()}>
    <doc-accordion></doc-accordion>
  </if>
  <if {useCombobox()}>
    <doc-combobox></doc-combobox>
  </if>
  <if {useThemePicker()}>
    <doc-theme-picker></doc-theme-picker>
  </if>
  <if {useResizable()}>
    <doc-resizable></doc-resizable>
  </if>
  <if {missingDocHost()}>
    <breadcrumb>
      <a href="/components">components</a>
      <small>{tag()}</small>
    </breadcrumb>
    <empty-state style(emptyEnter)>
      <strong slot="title">Docs host not wired for “{tag()}”</strong>
      Package meta is first-class (<code>doc-{tag()}</code>) but this shell has no
      mount for that CE yet.
    </empty-state>
    <row gap="2">
      <a variant="solid" tone="primary" href="/components">← All components</a>
    </row>
  </if>
  <if {!showViewer()}>
    <breadcrumb>
      <a href="/components">components</a>
      <small>{tag()}</small>
    </breadcrumb>
    <empty-state style(emptyEnter)>
      <strong slot="title">Nothing named “{tag()}” here</strong>
      Every element with a docs package has a detail page. Pack components
      from the catalog render here too — this looks like a typo.
    </empty-state>
    <row gap="2">
      <a variant="solid" tone="primary" href="/components">← All components</a>
    </row>
  </if>
</column>
