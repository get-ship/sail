<!--
  <doc-textarea> — first-class docs host for /components/textarea.
  Specimen left, knobs right + examples. Shared surface: component.field.*.
-->

<controller>
import type { Scope } from './doc-textarea.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const TA_SIZES = ['xs', 'sm', 'md', 'lg'] as const
  const sizeIdx = signal(2)
  const taInvalid = signal(false)
  const taDisabled = signal(false)
  const placeholder = signal('Tell us about yourself')
  const rowsIdx = signal(3)
  const bio = signal('')
  const taSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), TA_SIZES.length - 1)
    return TA_SIZES[i] ?? 'md'
  })
  const taRows = derived(() => {
    const n = Math.round(Number(rowsIdx()) || 3)
    return Math.min(12, Math.max(1, n))
  })
  const taMeta = derived(() => {
    const inv = taInvalid() ? 'invalid' : 'valid'
    const d = taDisabled() ? 'disabled' : 'enabled'
    return `${taSize()} · rows ${taRows()} · ${inv} · ${d} · component.field.*`
  })
  const entry = DETAILS.textarea
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const taExBind = () =>
    `// notes = signal('')\n<textarea rows="3" placeholder="Notes" bind:value={notes}></textarea>`
  const taExSizes = () =>
    `<textarea size="xs" rows="2" placeholder="xs"></textarea>\n<textarea size="sm" rows="2" placeholder="sm"></textarea>\n<textarea rows="2" placeholder="md"></textarea>\n<textarea size="lg" rows="2" placeholder="lg"></textarea>`
  const taExInvalid = () =>
    `<textarea invalid rows="3" placeholder="Required">too short</textarea>\n<!-- invalid → border(destructive) -->`
  const taExRest = () =>
    `<textarea rows="3" placeholder="Bio"></textarea>\n<!-- rest → component.field.bg / border / fg; use:autosize grows with content -->`
  return {
    sizeIdx,
    taInvalid,
    taDisabled,
    placeholder,
    rowsIdx,
    bio,
    taSize,
    taRows,
    taMeta,
    heading,
    blurb,
    props,
    usageCode,
    taExBind,
    taExSizes,
    taExInvalid,
    taExRest,
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
            <if {taDisabled()}>
              <if {taInvalid()}>
                <textarea
                  size={taSize()}
                  rows={taRows()}
                  placeholder={placeholder()}
                  value={bio()}
                  invalid
                  disabled
                  style(fullW)
                ></textarea>
              {else =>
                <textarea
                  size={taSize()}
                  rows={taRows()}
                  placeholder={placeholder()}
                  value={bio()}
                  disabled
                  style(fullW)
                ></textarea>}</if>
            {else =>
              <if {taInvalid()}>
                <textarea
                  size={taSize()}
                  rows={taRows()}
                  placeholder={placeholder()}
                  bind:value={bio}
                  invalid
                  style(fullW)
                ></textarea>
              {else =>
                <textarea
                  size={taSize()}
                  rows={taRows()}
                  placeholder={placeholder()}
                  bind:value={bio}
                  style(fullW)
                ></textarea>}</if>}</if>
            <p style(monoXs)>{taMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <row gap="2" align="center" justify="between">
              <label style(knobLabel)>Size</label>
              <span style(sizeValue)>{taSize()}</span>
            </row>
            <slider bind:value={sizeIdx} min="0" max="3" step="1" size="sm" label="Size"></slider>
            <row gap="0" align="center" style(tickRow)>
              <span style(tick)>xs</span>
              <span style(tick)>sm</span>
              <span style(tick)>md</span>
              <span style(tick)>lg</span>
            </row>
          </column>
          <column gap="2">
            <row gap="2" align="center" justify="between">
              <label style(knobLabel)>Rows</label>
              <span style(sizeValue)>{taRows()}</span>
            </row>
            <slider bind:value={rowsIdx} min="1" max="8" step="1" size="sm" label="Rows"></slider>
            <p style(mutedXs)>Initial height in lines (default <code>3</code>). <code>use:autosize</code> grows past this.</p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Placeholder</label>
            <input
              type="text"
              bind:value={placeholder}
              size="sm"
              placeholder="hint text"
            />
          </column>
          <column gap="2">
            <label style(knobLabel)>Invalid</label>
            <row gap="2" align="center">
              <switch bind:checked={taInvalid} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{taInvalid() ? 'destructive border' : 'rest border'}</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Disabled</label>
            <row gap="2" align="center">
              <switch bind:checked={taDisabled} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{taDisabled() ? 'not-allowed' : 'interactive'}</span>
            </row>
            <p style(mutedXs)>
              Rest paint from <code>component.field.*</code> — shared with input/select.
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
          Native multi-line field locked to a signal. Default <code>rows=3</code>.
        </p>
        <textarea rows="3" placeholder="Notes" bind:value={bio} style(fullW)></textarea>
        <code-block code={taExBind()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Size ladder</h3>
        <p style(exampleLead)>
          <code>xs</code> · <code>sm</code> · <code>md</code> · <code>lg</code> scale min-height + type.
        </p>
        <column gap="2" style(stageColMd)>
          <textarea size="xs" rows="2" placeholder="xs" style(fullW)></textarea>
          <textarea size="sm" rows="2" placeholder="sm" style(fullW)></textarea>
          <textarea rows="2" placeholder="md" style(fullW)></textarea>
          <textarea size="lg" rows="2" placeholder="lg" style(fullW)></textarea>
        </column>
        <code-block code={taExSizes()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Invalid</h3>
        <p style(exampleLead)>
          Compound <code>invalid</code> paints a destructive border on the shared field surface.
        </p>
        <textarea invalid rows="3" placeholder="Required" style(fullW)>too short</textarea>
        <code-block code={taExInvalid()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Rest tokens (themeable)</h3>
        <p style(exampleLead)>
          <code>component.field.bg</code> / <code>border</code> / <code>fg</code> — same leaves as
          input and select. <code>use:autosize</code> grows height with content.
        </p>
        <textarea rows="3" placeholder="Bio" style(fullW)></textarea>
        <code-block code={taExRest()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
