<!--
  <doc-kbd> — /components/kbd
  Shared chrome: split + snippet copy + usage Code|Live|Split.
  No size / variant / tone host props — compose keycaps in markup.
-->

<scope>
  style page = { box: size({ w: full, max-w: breakpoint-desktop }).space({ mx: auto }) }
  style mutedSm = { paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
  style mutedXs = { paint: ink({ color: muted-foreground }), text: type({ size: xs }) }
  style knobLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, weight: medium, tracking: wide, case: uppercase }),
}
  style exampleBlock = { box: size({ w: full }).space({ pad-t: 2, pad-b: 4 }) }
  style sectionLabel = { paint: ink({ color: muted-foreground }), text: type({ size: xs, tracking: wide, case: uppercase }) }
  style demoRow = { box: size({ w: full }).space({ pad-y: 2 }) }
</scope>

<controller>
import type { Scope } from './doc-kbd.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  // Docs-only content switch — not a host prop
  const chord = signal('search')
  const entry = DETAILS.kbd
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    switch (chord()) {
      case 'save':
        return `<kbd>⌘</kbd>\n<kbd>S</kbd>`
      case 'escape':
        return `<kbd>Esc</kbd>`
      default:
        return `<kbd>Ctrl</kbd> + <kbd>K</kbd>`
    }
  })

  const kbdExChord = () =>
    `Press <kbd>Ctrl</kbd> + <kbd>K</kbd> to open the command palette.`
  const kbdExMod = () =>
    `<kbd>⌘</kbd><kbd>⇧</kbd><kbd>P</kbd>`
  const kbdExKeys = () =>
    `<kbd>Esc</kbd>\n<kbd>Tab</kbd>\n<kbd>↵</kbd>`

  return {
    chord,
    specimenCode,
    heading,
    blurb,
    props,
    usageCode,
    kbdExChord,
    kbdExMod,
    kbdExKeys,
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
    <column slot="specimen" gap="4" align="center">
      <if {chord() === 'save'}>
        <row gap="1" align="center">
          <kbd>⌘</kbd>
          <kbd>S</kbd>
        </row>
      {else =>
        <if {chord() === 'escape'}>
          <kbd>Esc</kbd>
        {else =>
          <row gap="1" align="center">
            <kbd>Ctrl</kbd>
            <span style(mutedXs)>+</span>
            <kbd>K</kbd>
          </row>}</if>}</if>
    </column>

    <column slot="knobs" gap="5" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Demo chord</label>
        <select size="sm" bind:value={chord} label="Demo chord">
          <option value="search">Ctrl + K</option>
          <option value="save">⌘ S</option>
          <option value="escape">Esc</option>
        </select>
        <p style(mutedXs)>
          Docs-only content demo — not a host prop. Compose keycaps in markup.
        </p>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="center">
      <row gap="1" align="center">
        <kbd>Ctrl</kbd>
        <span style(mutedXs)>+</span>
        <kbd>K</kbd>
      </row>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <doc-example
        title="Command chord"
        lead="Pair keycaps with plain text — mono type, muted surface, physical bottom edge."
        code={kbdExChord()}
      >
        <p style(mutedSm)>
          Press <kbd>Ctrl</kbd> + <kbd>K</kbd> to open the command palette.
        </p>
      </doc-example>

      <doc-example
        title="Modifier stack"
        lead="Adjacent keycaps read as a chord without a size or tone API."
        code={kbdExMod()}
      >
        <row gap="1" align="center" style(demoRow)>
          <kbd>⌘</kbd>
          <kbd>⇧</kbd>
          <kbd>P</kbd>
        </row>
      </doc-example>

      <doc-example
        title="Common keys"
        lead="Single keycaps for Esc, Tab, and Enter."
        code={kbdExKeys()}
      >
        <row gap="1" align="center" style(demoRow)>
          <kbd>Esc</kbd>
          <kbd>Tab</kbd>
          <kbd>↵</kbd>
        </row>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
