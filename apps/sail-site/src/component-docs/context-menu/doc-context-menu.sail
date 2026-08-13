<!--
  <doc-context-menu> — /components/context-menu
  Shared chrome: split + snippet copy + usage Code|Live|Split.
  Specimen: for= target + onSelect live.
-->

<scope>
  style page = { box: size({ w: full, max-w: breakpoint-desktop }).space({ mx: auto }).overflow({ all: visible }) }
  style mutedXs = { paint: ink({ color: muted-foreground }), text: type({ size: xs }) }
  style monoXs = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style specimen = { box: space({ pad: 2 }).overflow({ all: visible }) }
  style knobLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, weight: medium, tracking: wide, case: uppercase }),
}
  style centerBox = { box: space({ pad: 4 }), paint: border({ color: border-subtle, width: 1 }).radius(md) }
  style exampleBlock = { box: size({ w: full }).space({ pad-t: 2, pad-b: 4 }).overflow({ all: visible }) }
  style sectionLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, tracking: wide, case: uppercase }),
}
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
      <p id="doc-cm-target" style(centerBox)>Right-click this card</p>
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
        <label style(knobLabel)>For</label>
        <p style(mutedXs)>
          <code>for</code> is the id of the right-click target (wired on specimen to
          <code>doc-cm-target</code>). Not a free-form knob — specimen owns the link.
        </p>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="center" style(specimen)>
      <p id="doc-cm-usage" style(centerBox)>right-click me</p>
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
        <p id="doc-cm-ex1" style(centerBox)>right-click me</p>
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
        <p id="doc-cm-ex2" style(centerBox)>right-click row</p>
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
