<!--
  /components/[tag] — thin detail shell.
  Docs SSOT: apps/sail-site/src/component-docs/<slug>/meta.ts
  Mounts exactly one docs host:
    - first-class package CE (doc-button, doc-badge, …) when meta.host is doc-*
    - or <doc-legacy-detail> only when meta.host is legacy / unmigrated
    - explicit missing-host empty state if first-class meta but no CE wired
    - empty state when no package / pack meta
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { docMeta, FIRST_CLASS_HOSTS } from '../../../src/compositions/component-docs-registry'
import { DETAILS, type DetailEntry } from '../../../src/compositions/component-details'
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
])

export default controller((ctx: ControllerScope<Scope>) => {
  const tag = derived(() => ctx.route.params.tag?.() ?? '')
  const meta = derived(() => docMeta(tag()))
  const entry = derived<DetailEntry | null>(() => DETAILS[tag()] ?? null)
  const knownDoc = derived(() => meta() !== null)
  const knownPack = derived(() => PACK_TAGS.has(tag()))
  const showViewer = derived(() => knownDoc() || knownPack())

  // FIRST_CLASS_HOSTS is SSOT from meta.host !== 'legacy'. Never route first-class → legacy.
  const firstClass = derived(() => FIRST_CLASS_HOSTS.has(tag()))
  const hasDocCe = derived(() => DOC_CE_SLUGS.has(tag()))
  const useDocHost = derived(() => firstClass() && hasDocCe())
  const missingDocHost = derived(() => firstClass() && !hasDocCe())
  // Legacy residual only when NOT first-class.
  const useLegacy = derived(() => showViewer() && !firstClass())

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
    useLegacy,
    heading,
    blurb,
    packLabel,
    packProps,
  }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style emptyEnter = { animate: enter({ opacity: 'fade', translate-y: 'spring:snappy' }) }
</scope>

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
  <if {missingDocHost()}>
    <breadcrumb>
      <a href="/components">components</a>
      <small>{tag()}</small>
    </breadcrumb>
    <empty-state style(emptyEnter)>
      <strong slot="title">Docs host not wired for “{tag()}”</strong>
      Package meta is first-class (<code>doc-{tag()}</code>) but this shell has no
      mount for that CE yet. Do not fall back to legacy residual after peel.
    </empty-state>
    <row gap="2">
      <a variant="solid" tone="primary" href="/components">← All components</a>
    </row>
  </if>
  <if {useLegacy()}>
    <doc-legacy-detail doc-tag={tag()}></doc-legacy-detail>
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
