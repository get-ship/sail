<!--
  /components/[tag] — detail route (E4 parse repair). Slim stage + viewer shell.
  Full configurator corpus restored separately; this file must parse+type-gate green.
-->

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).pad(4) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style emptyEnter = { animate: enter({ opacity: 'fade', translate-y: 'spring:snappy' }) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { DETAILS } from '../../../src/compositions/component-details'
import {
  PACK_PROPS,
  PACK_TAGS,
  packBlurbFor,
  packLabelFor,
} from '../../../src/compositions/component-pack-meta'

export default controller((ctx: ControllerScope<Scope>) => {
  const tag = derived(() => ctx.route.params.tag?.() ?? '')
  const entry = derived(() => {
    const t = tag()
    const list = Object.values(DETAILS) as Array<{ key: string; heading?: string; blurb?: string; props?: unknown[]; usage?: string; variants?: string[]; variantAttr?: string }>
    return list.find((e) => e.key === t) ?? null
  })
  const known = derived(() => entry() !== null)
  const knownPack = derived(() => PACK_TAGS.has(tag()))
  const showViewer = derived(() => known() || knownPack())
  const heading = derived(() => entry()?.heading ?? tag())
  const blurb = derived(() => entry()?.blurb ?? '')
  const props = derived(() => entry()?.props ?? [])
  const usageCode = derived(() => entry()?.usage ?? '')
  const packBlurb = derived(() => packBlurbFor(tag()))
  const packLabel = derived(() => packLabelFor(tag()))
  const packProps = derived(() => PACK_PROPS[tag()] ?? [])
  const viewerVariants = derived(() => entry()?.variants ?? [])
  const viewerVariantAttr = derived(() => entry()?.variantAttr ?? '')
  const onVariantChange = (_d: { variant?: string }) => {}
  return {
    tag,
    entry,
    known,
    knownPack,
    showViewer,
    heading,
    blurb,
    props,
    usageCode,
    packBlurb,
    packLabel,
    packProps,
    viewerVariants,
    viewerVariantAttr,
    onVariantChange,
  }
})
</controller>

<column gap="4" style(page)>
  <if {showViewer()}>
    <sail-component-viewer
      name={heading()}
      blurb={knownPack() ? packBlurb() : blurb()}
      pack={knownPack() ? packLabel() : 'Component primitives'}
      variant-attr={viewerVariantAttr()}
      usage={usageCode()}
      variants={viewerVariants()}
      rows={knownPack() ? packProps() : props()}
      on-variant-change={onVariantChange}
    >
      <column slot="stage" gap="3">
        <p style(mutedSm)>{heading()} — live stage. Props and usage are in the rail.</p>
      </column>
    </sail-component-viewer>
    <row gap="2">
      <a variant="default" href="/components">All components</a>
      <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
    </row>
  {else =>
    <column gap="4">
      <breadcrumb>
        <a href="/components">components</a>
        <small>{tag()}</small>
      </breadcrumb>
      <empty-state style(emptyEnter)>
        <strong slot="title">Nothing named here</strong>
        <p style(mutedSm)>Every element on the L1 surface has a detail page.</p>
      </empty-state>
      <row gap="2">
        <a variant="solid" tone="primary" href="/components">All components</a>
      </row>
    </column>
  }</if>
</column>
