<!--
  <doc-kbd> — first-class docs host for /components/kbd.
  Zero host Style axes — specimen + face copy; optional demo chord switch (demoOnly).
-->

<controller>
import type { Scope } from './doc-kbd.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  // demoOnly content switch — not a host prop
  const chord = signal('search')
  const entry = DETAILS.kbd
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const kbdMeta = derived(() => {
    switch (chord()) {
      case 'save':
        return '⌘ S · component.kbd.bg / border / fg'
      case 'escape':
        return 'Esc · component.kbd.bg / border / fg'
      default:
        return 'Ctrl + K · component.kbd.bg / border / fg'
    }
  })
  const kbdExChord = () =>
    `Press <kbd>Ctrl</kbd> + <kbd>K</kbd> to open the command palette.`
  const kbdExMod = () =>
    `<kbd>⌘</kbd><kbd>S</kbd>\n<!-- compose chords in markup — no size/variant/tone host props -->`
  const kbdExRest = () =>
    `<kbd>Esc</kbd>\n<!-- paint → component.kbd.bg · border · fg -->`
  return {
    chord,
    kbdMeta,
    heading,
    blurb,
    props,
    usageCode,
    kbdExChord,
    kbdExMod,
    kbdExRest,
  }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style stageSplit = { box: w(full).minW(0).minH(64) }
  style stageCanvas = { box: grow(1).basis(0).minW(0).minH(64).pad(8) }
  style knobPanel = {
    box: w(panel-max-w-sm).shrink(0).pad(5).minH(64),
    paint: borderLWidth(1).borderL(border-subtle),
  }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style exampleBlock = { box: w(full).padY(2) }
  style exampleH = { text: fontSize(md).fontWeight(semibold) }
  style exampleLead = { paint: fg(muted-foreground), text: fontSize(sm) }
  style sectionLabel = { paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) }
  style faceRow = { box: w(full) }
</scope>

<column gap="4" style(page)>
  <sail-component-viewer
    name={heading()}
    blurb={blurb()}
    pack="Component primitives"
    variant-attr=""
    usage={usageCode()}
    variants={[]}
    rows={props()}
  >
    <column slot="stage" gap="4" align="stretch" style({ box: w(full).minW(0) })>
      <row nowrap gap="0" align="stretch" style(stageSplit)>
        <center style(stageCanvas)>
          <column gap="4" align="center">
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
            <p style(monoXs)>{kbdMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
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
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <column gap="1" style(faceRow)>
              <p style(mutedXs)><code>component.kbd.bg</code> — fill</p>
              <p style(mutedXs)><code>component.kbd.border</code> — edge (+ doubled bottom)</p>
              <p style(mutedXs)><code>component.kbd.fg</code> — mono ink</p>
            </column>
            <p style(mutedXs)>No size / variant / tone host props. Theme the leaves; aesthetics beyond contrast are operator taste.</p>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <column gap="2">
        <h3 style(exampleH)>Command chord</h3>
        <p style(exampleLead)>
          Pair keycaps with plain text — mono type, muted surface, physical bottom edge.
        </p>
        <p style(mutedSm)>
          Press <kbd>Ctrl</kbd> + <kbd>K</kbd> to open the command palette.
        </p>
        <code-block code={kbdExChord()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Modifier stack</h3>
        <p style(exampleLead)>
          Adjacent keycaps read as a chord without a size or tone API.
        </p>
        <row gap="1" align="center">
          <kbd>⌘</kbd>
          <kbd>⇧</kbd>
          <kbd>P</kbd>
        </row>
        <code-block code={kbdExMod()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Rest tokens (themeable)</h3>
        <p style(exampleLead)>
          Paint from <code>component.kbd.bg</code> · <code>border</code> · <code>fg</code>.
        </p>
        <row gap="1" align="center">
          <kbd>Esc</kbd>
          <kbd>Tab</kbd>
          <kbd>↵</kbd>
        </row>
        <code-block code={kbdExRest()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
