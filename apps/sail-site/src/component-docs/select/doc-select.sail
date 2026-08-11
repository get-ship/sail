<!--
  <doc-select> — first-class docs host for /components/select.
  Specimen left, knobs right + examples. Shared surface: component.field.*.
  Native closed picker only — not combobox.
-->

<controller>
import type { Scope } from './doc-select.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const SEL_SIZES = ['xs', 'sm', 'md', 'lg'] as const
  const sizeIdx = signal(2)
  const selInvalid = signal(false)
  const selDisabled = signal(false)
  const selMultiple = signal(false)
  const team = signal('design')
  const selSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), SEL_SIZES.length - 1)
    return SEL_SIZES[i] ?? 'md'
  })
  const selMeta = derived(() => {
    const inv = selInvalid() ? 'invalid' : 'valid'
    const d = selDisabled() ? 'disabled' : 'enabled'
    const m = selMultiple() ? 'multiple' : 'single'
    return `${selSize()} · ${m} · ${inv} · ${d} · component.field.*`
  })
  const entry = DETAILS.select
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const selExBind = () =>
    `// team = signal('design')\n<select bind:value={team}>\n  <option value="design">Design</option>\n  <option value="eng">Engineering</option>\n</select>`
  const selExSizes = () =>
    `<select size="xs">…</select>\n<select size="sm">…</select>\n<select>…</select>\n<select size="lg">…</select>`
  const selExInvalid = () =>
    `<select invalid>\n  <option value="">Pick one</option>\n  <option value="a">A</option>\n</select>\n<!-- invalid → border(destructive) -->`
  const selExMultiple = () =>
    `<select multiple>\n  <option>Ada</option>\n  <option>Grace</option>\n</select>\n<!-- native multi-select; open list stays the OS picker -->`
  const selExRest = () =>
    `<select>\n  <option>Alan</option>\n</select>\n<!-- rest → component.field.*; chevron via SELECT_MAP padE -->`
  return {
    sizeIdx,
    selInvalid,
    selDisabled,
    selMultiple,
    team,
    selSize,
    selMeta,
    heading,
    blurb,
    props,
    usageCode,
    selExBind,
    selExSizes,
    selExInvalid,
    selExMultiple,
    selExRest,
  }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style stageCol = { box: w(panel-max-w-sm).maxW(full) }
  style stageColMd = { box: w(panel-max-w-md).maxW(full) }
  style fullW = { box: w(full) }
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
  style tickRow = { box: w(full).padT(1) }
  style tick = {
    box: grow(1).basis(0).minW(0),
    paint: fg(muted-foreground),
    text: fontSize(xs).fontFamily(mono).textAlign(center),
  }
  style exampleBlock = { box: w(full).padY(2) }
  style exampleH = { text: fontSize(md).fontWeight(semibold) }
  style exampleLead = { paint: fg(muted-foreground), text: fontSize(sm) }
  style sectionLabel = { paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) }
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
          <column gap="4" align="center" style(stageColMd)>
            <if {selMultiple()}>
              <if {selDisabled()}>
                <if {selInvalid()}>
                  <select size={selSize()} multiple invalid disabled style(fullW)>
                    <option value="design">Design</option>
                    <option value="eng">Engineering</option>
                    <option value="research">Research</option>
                  </select>
                {else =>
                  <select size={selSize()} multiple disabled style(fullW)>
                    <option value="design">Design</option>
                    <option value="eng">Engineering</option>
                    <option value="research">Research</option>
                  </select>}</if>
              {else =>
                <if {selInvalid()}>
                  <select size={selSize()} multiple invalid style(fullW)>
                    <option value="design">Design</option>
                    <option value="eng">Engineering</option>
                    <option value="research">Research</option>
                  </select>
                {else =>
                  <select size={selSize()} multiple style(fullW)>
                    <option value="design">Design</option>
                    <option value="eng">Engineering</option>
                    <option value="research">Research</option>
                  </select>}</if>}</if>
            {else =>
              <if {selDisabled()}>
                <if {selInvalid()}>
                  <select size={selSize()} value={team()} invalid disabled style(fullW)>
                    <option value="design">Design</option>
                    <option value="eng">Engineering</option>
                    <option value="research">Research</option>
                  </select>
                {else =>
                  <select size={selSize()} value={team()} disabled style(fullW)>
                    <option value="design">Design</option>
                    <option value="eng">Engineering</option>
                    <option value="research">Research</option>
                  </select>}</if>
              {else =>
                <if {selInvalid()}>
                  <select size={selSize()} bind:value={team} invalid style(fullW)>
                    <option value="design">Design</option>
                    <option value="eng">Engineering</option>
                    <option value="research">Research</option>
                  </select>
                {else =>
                  <select size={selSize()} bind:value={team} style(fullW)>
                    <option value="design">Design</option>
                    <option value="eng">Engineering</option>
                    <option value="research">Research</option>
                  </select>}</if>}</if>}</if>
            <p style(monoXs)>{selMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <row gap="2" align="center" justify="between">
              <label style(knobLabel)>Size</label>
              <span style(sizeValue)>{selSize()}</span>
            </row>
            <slider bind:value={sizeIdx} min="0" max="3" step="1" size="sm" label="Size"></slider>
            <row gap="0" align="center" style(tickRow)>
              <span style(tick)>xs</span>
              <span style(tick)>sm</span>
              <span style(tick)>md</span>
              <span style(tick)>lg</span>
            </row>
            <p style(mutedXs)>
              Control height + type scale. <code>padE</code> reserved for the chevron glyph.
            </p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Invalid</label>
            <row gap="2" align="center">
              <switch bind:checked={selInvalid} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{selInvalid() ? 'destructive border' : 'rest border'}</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Disabled</label>
            <row gap="2" align="center">
              <switch bind:checked={selDisabled} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{selDisabled() ? 'not-allowed' : 'interactive'}</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Multiple</label>
            <row gap="2" align="center">
              <switch bind:checked={selMultiple} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{selMultiple() ? 'multi' : 'single'}</span>
            </row>
            <p style(mutedXs)>
              Native multi-select attr — no PE chrome change. Open list stays the OS picker,
              not a styled listbox (<code>combobox</code>).
            </p>
          </column>
          <column gap="2">
            <p style(mutedXs)>
              Rest from <code>component.field.*</code> shared with input/textarea.
            </p>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <column gap="2">
        <h3 style(exampleH)>bind:value</h3>
        <p style(exampleLead)>
          Platform keyboard, typeahead, and mobile sheets. Pair with
          <code>&lt;option&gt;</code> / <code>&lt;optgroup&gt;</code> children.
        </p>
        <select bind:value={team} style(fullW)>
          <option value="design">Design</option>
          <option value="eng">Engineering</option>
          <option value="research">Research</option>
        </select>
        <code-block code={selExBind()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Size ladder</h3>
        <p style(exampleLead)>
          <code>xs</code> · <code>sm</code> · <code>md</code> · <code>lg</code> — same channel as input.
        </p>
        <column gap="2" style(stageColMd)>
          <select size="xs" style(fullW)>
            <option>Ada</option><option>Grace</option>
          </select>
          <select size="sm" style(fullW)>
            <option>Ada</option><option>Grace</option>
          </select>
          <select style(fullW)>
            <option>Ada</option><option>Grace</option>
          </select>
          <select size="lg" style(fullW)>
            <option>Ada</option><option>Grace</option>
          </select>
        </column>
        <code-block code={selExSizes()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Invalid</h3>
        <p style(exampleLead)>PE attr — paints destructive border like input/textarea.</p>
        <select invalid style(fullW)>
          <option value="">Pick one</option>
          <option value="a">A</option>
        </select>
        <code-block code={selExInvalid()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Multiple</h3>
        <p style(exampleLead)>
          Native multi-select. Still not a combobox — arbitrary item chrome is a different L1.
        </p>
        <select multiple style(fullW)>
          <option>Ada</option>
          <option>Grace</option>
          <option>Alan</option>
        </select>
        <code-block code={selExMultiple()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Rest tokens (themeable)</h3>
        <p style(exampleLead)>
          <code>component.field.*</code> surface; chevron room via <code>padE(8)</code>.
        </p>
        <select style(fullW)>
          <option>Alan</option>
          <option>Grace</option>
        </select>
        <code-block code={selExRest()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
