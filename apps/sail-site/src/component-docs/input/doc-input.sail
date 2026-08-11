<!--
  <doc-input> — first-class docs host for /components/input.
  Specimen left, knobs right (Mantine-style configurator) + examples walkthrough.
  Shared surface: component.field.* (no component.input.* invent).
-->

<controller>
import type { Scope } from './doc-input.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const INPUT_SIZES = ['xs', 'sm', 'md', 'lg'] as const
  const sizeIdx = signal(2)
  const inputType = signal('text')
  const inputInvalid = signal(false)
  const inputDisabled = signal(false)
  const placeholder = signal('you@ship.dev')
  const textVal = signal('ada@ship.dev')
  const checkedVal = signal(true)
  const rangeVal = signal(40)
  const inputSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), INPUT_SIZES.length - 1)
    return INPUT_SIZES[i] ?? 'md'
  })
  const isCheckLike = derived(() => {
    const t = inputType()
    return t === 'checkbox' || t === 'radio'
  })
  const isRange = derived(() => inputType() === 'range')
  const isHidden = derived(() => inputType() === 'hidden')
  const inputMeta = derived(() => {
    const inv = inputInvalid() ? 'invalid' : 'valid'
    const d = inputDisabled() ? 'disabled' : 'enabled'
    return `${inputType()} · ${inputSize()} · ${inv} · ${d} · component.field.*`
  })
  const entry = DETAILS.input
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const inExBind = () =>
    `// email = signal('')\n<input type="email" placeholder="you@ship.dev" bind:value={email} />`
  const inExSizes = () =>
    `<input size="xs" placeholder="xs" />\n<input size="sm" placeholder="sm" />\n<input placeholder="md" />\n<input size="lg" placeholder="lg" />`
  const inExInvalid = () =>
    `<input type="email" value="not-an-email" invalid />\n<!-- invalid → border(destructive); mirror on <field invalid> for hint/error -->`
  const inExCheck = () =>
    `// agreed = signal(true)\n<label>\n  <input type="checkbox" bind:checked={agreed} /> Email me updates\n</label>\n<!-- no <checkbox> tag — checkSized skin on input -->`
  const inExRest = () =>
    `<input placeholder="Display name" />\n<!-- rest → component.field.bg / border / fg; no filled/unstyled invent -->`
  return {
    sizeIdx,
    inputType,
    inputInvalid,
    inputDisabled,
    placeholder,
    textVal,
    checkedVal,
    rangeVal,
    inputSize,
    isCheckLike,
    isRange,
    isHidden,
    inputMeta,
    heading,
    blurb,
    props,
    usageCode,
    inExBind,
    inExSizes,
    inExInvalid,
    inExCheck,
    inExRest,
  }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style stageCol = { box: w(panel-max-w-sm).maxW(full) }
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
          <column gap="4" align="center" style(stageCol)>
            <if {isHidden()}>
              <p style(mutedSm)>
                type=<code>hidden</code> — no paint (display none). PE still accepts size/invalid/disabled.
              </p>
              <input type="hidden" value="token" size={inputSize()} />
            {else =>
              <if {isCheckLike()}>
                <row gap="3" align="center">
                  <if {inputDisabled()}>
                    <if {inputInvalid()}>
                      <input type={inputType()} bind:checked={checkedVal} size={inputSize()} invalid disabled />
                    {else =>
                      <input type={inputType()} bind:checked={checkedVal} size={inputSize()} disabled />}</if>
                  {else =>
                    <if {inputInvalid()}>
                      <input type={inputType()} bind:checked={checkedVal} size={inputSize()} invalid />
                    {else =>
                      <input type={inputType()} bind:checked={checkedVal} size={inputSize()} />}</if>}</if>
                  <p style(monoXs)>{checkedVal() ? 'checked' : 'unchecked'}</p>
                </row>
              {else =>
                <if {isRange()}>
                  <if {inputDisabled()}>
                    <input type="range" value={rangeVal()} min="0" max="100" size={inputSize()} style(fullW) disabled />
                  {else =>
                    <input type="range" bind:value={rangeVal} min="0" max="100" size={inputSize()} style(fullW) />}</if>
                {else =>
                  <if {inputDisabled()}>
                    <if {inputInvalid()}>
                      <input type={inputType()} value={textVal()} placeholder={placeholder()} size={inputSize()} style(fullW) invalid disabled />
                    {else =>
                      <input type={inputType()} value={textVal()} placeholder={placeholder()} size={inputSize()} style(fullW) disabled />}</if>
                  {else =>
                    <if {inputInvalid()}>
                      <input type={inputType()} bind:value={textVal} placeholder={placeholder()} size={inputSize()} style(fullW) invalid />
                    {else =>
                      <input type={inputType()} bind:value={textVal} placeholder={placeholder()} size={inputSize()} style(fullW) />}</if>}</if>}</if>}</if>}</if>
            <p style(monoXs)>{inputMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Type</label>
            <select size="sm" bind:value={inputType} label="Type">
              <option value="text">text</option>
              <option value="email">email</option>
              <option value="password">password</option>
              <option value="number">number</option>
              <option value="search">search</option>
              <option value="tel">tel</option>
              <option value="url">url</option>
              <option value="date">date</option>
              <option value="time">time</option>
              <option value="datetime-local">datetime-local</option>
              <option value="month">month</option>
              <option value="week">week</option>
              <option value="file">file</option>
              <option value="color">color</option>
              <option value="checkbox">checkbox</option>
              <option value="radio">radio</option>
              <option value="range">range</option>
              <option value="submit">submit</option>
              <option value="hidden">hidden</option>
            </select>
            <p style(mutedXs)>
              Type gates the skin. Text-like → <code>component.field.*</code>;
              checkbox/radio → checkSized/radioSized (no separate tags).
            </p>
          </column>
          <column gap="2">
            <row gap="2" align="center" justify="between">
              <label style(knobLabel)>Size</label>
              <span style(sizeValue)>{inputSize()}</span>
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
            <label style(knobLabel)>Placeholder</label>
            <input
              type="text"
              bind:value={placeholder}
              size="sm"
              placeholder="hint text"
            />
            <p style(mutedXs)>Hint text for empty text-like fields (ignored on checkbox/radio/range/hidden).</p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Invalid</label>
            <row gap="2" align="center">
              <switch bind:checked={inputInvalid} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{inputInvalid() ? 'destructive border' : 'rest border'}</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Disabled</label>
            <row gap="2" align="center">
              <switch bind:checked={inputDisabled} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{inputDisabled() ? 'not-allowed' : 'interactive'}</span>
            </row>
            <p style(mutedXs)>
              Native <code>disabled</code> + PE opacity. Rest paint from
              <code>component.field.*</code> — no <code>component.input.*</code> leaves.
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
          Two-way signal lock for value-bearing types. Pair with <code>type</code> for the skin.
        </p>
        <input type="email" placeholder="you@ship.dev" bind:value={textVal}  style(fullW) />
        <code-block code={inExBind()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Size ladder</h3>
        <p style(exampleLead)>
          <code>xs</code> · <code>sm</code> · <code>md</code> · <code>lg</code> scale control height
          (default <code>md</code>).
        </p>
        <column gap="2" style(stageCol)>
          <input size="xs" placeholder="xs"  style(fullW) />
          <input size="sm" placeholder="sm"  style(fullW) />
          <input placeholder="md"  style(fullW) />
          <input size="lg" placeholder="lg"  style(fullW) />
        </column>
        <code-block code={inExSizes()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Invalid</h3>
        <p style(exampleLead)>
          Compound <code>invalid</code> paints a destructive border. Mirror the same attr on
          the field host when using hint/error slots.
        </p>
        <input type="email" value="not-an-email" style(fullW) invalid />
        <code-block code={inExInvalid()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Checkbox type (no checkbox tag)</h3>
        <p style(exampleLead)>
          Catalog alias only — real control is input with type=checkbox and checkSized skin.
          See also the <a href="/components/checkbox">checkbox hub tile</a>.
        </p>
        <row gap="2" align="center">
          <input type="checkbox" bind:checked={checkedVal} />
          <span style(mutedSm)>Email me product updates</span>
        </row>
        <code-block code={inExCheck()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Rest tokens (themeable)</h3>
        <p style(exampleLead)>
          Text-like fields share <code>component.field.bg</code> / <code>border</code> /
          <code>fg</code> with select and textarea. No filled/unstyled host invent.
        </p>
        <input placeholder="Display name"  style(fullW) />
        <code-block code={inExRest()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
