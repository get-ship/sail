<!--
  <doc-toggle-group> — first-class docs host for /components/toggle-group.
  Segmented control host — type × value × variant × orientation.
  Keep <toggle> as leaf cells (no top-level toggle package).
-->

<controller>
import type { Scope } from './doc-toggle-group.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const tgType = signal('single')
  const tgVariant = signal('outline')
  const tgOrientation = signal('horizontal')
  const tgValue = signal('day')
  const lastChange = signal('')
  const onToggleChange = (detail: { value?: string }): void => {
    const v = detail?.value ?? ''
    tgValue.set(v)
    lastChange.set(v)
  }
  const tgMeta = derived(() => {
    const ch = lastChange() ? ` · last ${lastChange()}` : ''
    return `${tgType()} · ${tgVariant()} · ${tgOrientation()} · value "${tgValue()}"${ch} · rail toggle-group.bg/border`
  })
  const entry = DETAILS['toggle-group']
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const tgExSingle = () =>
    `<toggle-group type="single" value="day" variant="outline" on-toggle-change={onChange}>\n  <toggle value="day">Day</toggle>\n  <toggle value="week">Week</toggle>\n  <toggle value="month">Month</toggle>\n</toggle-group>`
  const tgExSegmented = () =>
    `<toggle-group variant="segmented" value="list">\n  <toggle value="grid">Grid</toggle>\n  <toggle value="list">List</toggle>\n  <toggle value="board">Board</toggle>\n</toggle-group>`
  const tgExMulti = () =>
    `<toggle-group type="multiple" value="grid board">\n  <toggle value="grid">Grid</toggle>\n  <toggle value="list">List</toggle>\n  <toggle value="board">Board</toggle>\n</toggle-group>`
  return {
    tgType,
    tgVariant,
    tgOrientation,
    tgValue,
    lastChange,
    onToggleChange,
    tgMeta,
    heading,
    blurb,
    props,
    usageCode,
    tgExSingle,
    tgExSegmented,
    tgExMulti,
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
          <column gap="3" align="center" style(stageCol)>
            <toggle-group
              type={tgType()}
              value={tgValue()}
              variant={tgVariant()}
              orientation={tgOrientation()}
              on-toggle-change={onToggleChange}
              label="Range"
            >
              <toggle value="day">Day</toggle>
              <toggle value="week">Week</toggle>
              <toggle value="month">Month</toggle>
            </toggle-group>
            <p style(monoXs)>{tgMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Type</label>
            <select size="sm" bind:value={tgType} label="Type">
              <option value="single">single</option>
              <option value="multiple">multiple</option>
            </select>
            <p style(mutedXs)>single = exclusive; multiple = space-joined value set.</p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Variant</label>
            <select size="sm" bind:value={tgVariant} label="Variant">
              <option value="outline">outline</option>
              <option value="ghost">ghost</option>
              <option value="segmented">segmented</option>
            </select>
            <p style(mutedXs)>
              <code>segmented</code> is the rail look — host paints
              <code>component.toggle-group.*</code>.
            </p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Orientation</label>
            <select size="sm" bind:value={tgOrientation} label="Orientation">
              <option value="horizontal">horizontal</option>
              <option value="vertical">vertical</option>
            </select>
          </column>
          <column gap="2">
            <label style(knobLabel)>Value</label>
            <select size="sm" bind:value={tgValue} label="Value">
              <option value="day">day</option>
              <option value="week">week</option>
              <option value="month">month</option>
              <option value="day week">day week</option>
              <option value="day month">day month</option>
              <option value="week month">week month</option>
              <option value="day week month">day week month</option>
            </select>
            <p style(mutedXs)>
              Selected value(s). Multi uses space-separated tokens. Live clicks also update via
              <code>onToggleChange</code>.
            </p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <column gap="1" style(faceRow)>
              <p style(mutedXs)><code>component.toggle-group.bg</code> — rail fill</p>
              <p style(mutedXs)><code>component.toggle-group.border</code> — rail edge</p>
              <p style(mutedXs)>
                Cell paint lives on <code>component.toggle.*</code> — not host faces.
              </p>
            </column>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <column gap="2">
        <h3 style(exampleH)>Single selection</h3>
        <p style(exampleLead)>
          Default type. Roving focus + <code>aria-pressed</code> on each <code>toggle</code>.
        </p>
        <toggle-group type="single" value="day" variant="outline" on-toggle-change={onToggleChange}>
          <toggle value="day">Day</toggle>
          <toggle value="week">Week</toggle>
          <toggle value="month">Month</toggle>
        </toggle-group>
        <code-block code={tgExSingle()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Segmented rail</h3>
        <p style(exampleLead)>
          <code>variant="segmented"</code> — the segmented control host look.
        </p>
        <toggle-group variant="segmented" value="list" on-toggle-change={onToggleChange}>
          <toggle value="grid">Grid</toggle>
          <toggle value="list">List</toggle>
          <toggle value="board">Board</toggle>
        </toggle-group>
        <code-block code={tgExSegmented()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Multiple</h3>
        <p style(exampleLead)>
          <code>type="multiple"</code> accumulates a space-joined value set.
        </p>
        <toggle-group type="multiple" value="grid board" on-toggle-change={onToggleChange}>
          <toggle value="grid">Grid</toggle>
          <toggle value="list">List</toggle>
          <toggle value="board">Board</toggle>
        </toggle-group>
        <code-block code={tgExMulti()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
