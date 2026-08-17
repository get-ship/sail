<!--
  <doc-context-menu> — /components/context-menu
  Shared chrome: split + snippet copy + usage Code|Live|Split.
  Specimen: for= target + onSelect live.
-->

<scope>
  style page = { size(max).overflow(visible) }
  style mutedXs = { fg(muted).type(body-sm) }
  style monoXs = { fg(muted).type(mono-sm) }
  style specimen = { overflow(visible) }
  style exampleBlock = { size(max).overflow(visible) }
  style sectionLabel = { fg(muted).type(label) }
</scope>

<controller>
import type { Scope } from './doc-context-menu.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const last = signal('')
  const onSelect = (detail: { value?: string }): void => {
    last.set(detail?.value ?? '')
  }

  const entry = DETAILS['context-menu']
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    return `<p id="canvas">right-click me</p>\n<context-menu for="canvas" on-select={onSelect}>\n  <menu-item value="cut">Cut</menu-item>\n  <menu-item value="copy">Copy</menu-item>\n</context-menu>`
  })

  const cmExBasic = () =>
    `<p id="canvas">right-click me</p>\n<context-menu for="canvas" on-select={onSelect}>\n  <menu-item value="cut">Cut</menu-item>\n  <menu-item value="copy">Copy</menu-item>\n</context-menu>`
  const cmExMore = () =>
    `<context-menu for="row">\n  <menu-item value="open">Open</menu-item>\n  <menu-item value="del">Delete</menu-item>\n</context-menu>`

  return {
    last,
    onSelect,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    cmExBasic,
    cmExMore,
  }
})
</controller>

<column gap="6" style(page)>
  <sail-component-viewer
    name={heading()}
    blurb={blurb()}
    pack="Component primitives"
    layout="split"
    snippet={specimenCode()}
    usage-live
    usage={usageCode()}
    rows={props()}
  >
    <column slot="specimen" gap="3" align="center" style(specimen)>
      <card id="doc-cm-target">Right-click this card</card>
      <context-menu for="doc-cm-target" id="doc-cm-live" on-select={onSelect}>
        <menu-item value="copy">Copy id</menu-item>
        <menu-item value="delete">Delete</menu-item>
      </context-menu>
      <if {last()}>
        <p style(monoXs)>selected {last()}</p>
      </if>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label>For</label>
        <p style(mutedXs)>
          <code>for</code> is the id of the right-click target (wired on specimen to
          <code>doc-cm-target</code>). Not a free-form knob — specimen owns the link.
        </p>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="center" style(specimen)>
      <card id="doc-cm-usage">right-click me</card>
      <context-menu for="doc-cm-usage" on-select={onSelect}>
        <menu-item value="cut">Cut</menu-item>
        <menu-item value="copy">Copy</menu-item>
      </context-menu>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Canvas target"
        lead="Opens at the pointer on contextmenu of for."
        code={cmExBasic()}
      >
        <card id="doc-cm-ex1">right-click me</card>
        <context-menu for="doc-cm-ex1" on-select={onSelect}>
          <menu-item value="cut">Cut</menu-item>
          <menu-item value="copy">Copy</menu-item>
        </context-menu>
      </doc-example>
      <doc-example
        title="Row actions"
        lead="Attach a menu to any row or list item via for."
        code={cmExMore()}
      >
        <card id="doc-cm-ex2">right-click row</card>
        <context-menu for="doc-cm-ex2">
          <menu-item value="open">Open</menu-item>
          <menu-item value="del">Delete</menu-item>
        </context-menu>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
