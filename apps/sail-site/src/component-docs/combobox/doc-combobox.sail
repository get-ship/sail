<!--
  <doc-combobox> — first-class docs host for /components/combobox.
  Specimen left, knobs right — placeholder · value (onSelect live).
-->

<controller>
import type { Scope } from './doc-combobox.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const placeholder = signal('Pick a framework — type to filter')
  const value = signal('')
  const last = signal('')
  const onSelect = (detail: { value?: string; label?: string }): void => {
    value.set(detail?.value ?? '')
    last.set(detail?.label ?? detail?.value ?? '')
  }
  const cbMeta = derived(() => {
    const v = value() ? ` · value "${value()}"` : ''
    const l = last() ? ` · picked ${last()}` : ''
    return `placeholder ·${v}${l} · component.combobox.bg/fg`
  })
  const entry = DETAILS.combobox
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const cbExBasic = () =>
    `<combobox placeholder="Pick a framework" on-select={onSelect}>\n  <option value="sail">Sail</option>\n  <option value="react">React</option>\n</combobox>`
  const cbExValue = () =>
    `<combobox value="sail" placeholder="Framework">\n  <option value="sail">Sail</option>\n  <option value="svelte">Svelte</option>\n</combobox>`
  const cbExFilter = () =>
    `// Type to filter · arrows rove · Enter commits · Escape closes\n<combobox placeholder="Fruit">…</combobox>`
  return {
    placeholder, value, last, onSelect, cbMeta,
    heading, blurb, props, usageCode,
    cbExBasic, cbExValue, cbExFilter,
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
  style stageCol = { box: w(panel-max-w-md).maxW(full) }
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
  style fullW = { box: w(full) }

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
          <column gap="3" align="stretch" style(stageCol)>
            <combobox
              placeholder={placeholder()}
              value={value()}
              on-select={onSelect}
              style(fullW)
            >
              <option value="sail">Sail</option>
              <option value="react">React</option>
              <option value="svelte">Svelte</option>
              <option value="solid">Solid</option>
            </combobox>
            <p style(monoXs)>{cbMeta()}</p>
            <p style(mutedXs)>Focus opens the full list · type filters · Enter commits.</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Placeholder</label>
            <select size="sm" bind:value={placeholder} label="Placeholder">
              <option value="Pick a framework — type to filter">Pick a framework…</option>
              <option value="Search packages">Search packages</option>
              <option value="Choose fruit">Choose fruit</option>
            </select>
          </column>
          <column gap="2">
            <label style(knobLabel)>Value</label>
            <select size="sm" bind:value={value} label="Value">
              <option value="">(none)</option>
              <option value="sail">sail</option>
              <option value="react">react</option>
              <option value="svelte">svelte</option>
              <option value="solid">solid</option>
            </select>
            <p style(mutedXs)>Committed value reflected on the host after select.</p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <column gap="1" style(faceRow)>
              <p style(mutedXs)><code>component.combobox.bg / fg</code> (listbox)</p>
              <p style(mutedXs)>Field chrome uses free globals (matches bare input).</p>
            </column>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Filterable list</h3>
        <p style(exampleLead)>Plain <code>option</code> children; on-select gets value + label.</p>
        <combobox placeholder="Pick a framework" on-select={onSelect} style(fullW)>
          <option value="sail">Sail</option>
          <option value="react">React</option>
          <option value="svelte">Svelte</option>
        </combobox>
        <code-block code={cbExBasic()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Pre-committed value</h3>
        <p style(exampleLead)><code>value</code> reflects the selected option.</p>
        <code-block code={cbExValue()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Keyboard</h3>
        <p style(exampleLead)>Arrows rove filtered set; Enter commits; Escape closes.</p>
        <code-block code={cbExFilter()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
