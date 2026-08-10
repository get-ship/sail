<!--
  /components/[tag] — thin detail shell.
  Docs SSOT: apps/sail-site/src/component-docs/<slug>/meta.ts
  Mounts exactly one docs host:
    - first-class package CE (doc-button, doc-badge, doc-fieldset, …)
    - or <doc-legacy-detail> for unmigrated packages
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

export default controller((ctx: ControllerScope<Scope>) => {
  const tag = derived(() => ctx.route.params.tag?.() ?? '')
  const meta = derived(() => docMeta(tag()))
  const entry = derived<DetailEntry | null>(() => DETAILS[tag()] ?? null)
  const knownDoc = derived(() => meta() !== null)
  const knownPack = derived(() => PACK_TAGS.has(tag()))
  const showViewer = derived(() => knownDoc() || knownPack())

  // First-class packages own their full detail body (viewer + stage + examples).
  const useButton = derived(() => tag() === 'button' && FIRST_CLASS_HOSTS.has('button'))
  const useBadge = derived(() => tag() === 'badge' && FIRST_CLASS_HOSTS.has('badge'))
  const useFieldset = derived(() => tag() === 'fieldset' && FIRST_CLASS_HOSTS.has('fieldset'))
  const useFirstClass = derived(() => useButton() || useBadge() || useFieldset())
  // Everything else that is known goes through the legacy residual host.
  const useLegacy = derived(() => showViewer() && !useFirstClass())

  // Shell-level empty-state chrome only (first-class + legacy own their footers).
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
    useButton,
    useBadge,
    useFieldset,
    useFirstClass,
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
