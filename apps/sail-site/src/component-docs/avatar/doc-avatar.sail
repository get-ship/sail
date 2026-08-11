<!--
  <doc-avatar> — first-class docs host for /components/avatar.
  Specimen left, knobs right — size · shape · tone · name/initials.
-->

<controller>
import type { Scope } from './doc-avatar.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const size = signal('md')
  const shape = signal('round')
  const avTone = signal('')
  const name = signal('Ada Lovelace')
  const initials = signal('')
  const sizeIdx = derived(() => {
    const order = ['xs', 'sm', 'md', 'lg', 'xl']
    return String(order.indexOf(size()))
  })
  const onSize = (v: string | number): void => {
    const order = ['xs', 'sm', 'md', 'lg', 'xl']
    const i = Math.max(0, Math.min(4, Number(v)))
    size.set(order[i] ?? 'md')
  }
  const pickTone = (t: string): void => avTone.set(t)
  const clearTone = (): void => avTone.set('')
  const avMeta = derived(() => {
    const t = avTone() || 'rest gradient'
    const id = initials() || name() || 'empty'
    return `${size()} · ${shape()} · ${t} · ${id} · component.avatar.bg/fg`
  })
  const entry = DETAILS.avatar
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const avExName = () =>
    `<avatar name="Ada Lovelace"></avatar>\n<avatar initials="GH" tone="primary"></avatar>`
  const avExShape = () =>
    `<avatar name="Alan Turing" size="lg" shape="square"></avatar>`
  const avExSizes = () =>
    `<row gap="2">\n  <avatar size="xs" initials="XS"></avatar>\n  <avatar size="sm" initials="SM"></avatar>\n  <avatar size="md" initials="MD"></avatar>\n  <avatar size="lg" initials="LG"></avatar>\n  <avatar size="xl" initials="XL"></avatar>\n</row>`
  return {
    size, shape, avTone, name, initials, sizeIdx, onSize, pickTone, clearTone, avMeta,
    heading, blurb, props, usageCode, avExName, avExShape, avExSizes,
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
  style swatchRow = { box: w(full) }
  style swatch = {
    box: size(6).shrink(0),
    paint: radius(full).borderWidth(1).border(border).cursor(pointer),
  }
  style swatchRest = { paint: bg(surface-1) }
  style swatchPrimary = { paint: bg(primary) }
  style swatchInfo = { paint: bg(info) }
  style swatchSuccess = { paint: bg(success) }
  style swatchWarning = { paint: bg(warning) }
  style swatchDestructive = { paint: bg(destructive) }
  style swatchAccent = { paint: bg(accent) }
  style swatchMuted = { paint: bg(muted) }
  style centerBox = { box: pad(4), paint: borderWidth(1).border(border-subtle).radius(md) }

  style tickRow = { box: w(full) }
  style tick = { paint: fg(muted-foreground), text: fontSize(xs), box: grow(1) }
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
          <column gap="3" align="center">
            <if {avTone()}>
              <if {initials()}>
                <avatar size={size()} shape={shape()} tone={avTone()} initials={initials()}></avatar>
              {else =>
                <avatar size={size()} shape={shape()} tone={avTone()} name={name()}></avatar>}</if>
            {else =>
              <if {initials()}>
                <avatar size={size()} shape={shape()} initials={initials()}></avatar>
              {else =>
                <avatar size={size()} shape={shape()} name={name()}></avatar>}</if>}</if>
            <p style(monoXs)>{avMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <row gap="2" align="center" justify="between">
              <label style(knobLabel)>Size</label>
              <span style(sizeValue)>{size()}</span>
            </row>
            <slider value={sizeIdx()} min="0" max="4" step="1" size="sm" label="Size" on-value-change={onSize}></slider>
            <row gap="0" align="center" style(tickRow)>
              <span style(tick)>xs</span>
              <span style(tick)>sm</span>
              <span style(tick)>md</span>
              <span style(tick)>lg</span>
              <span style(tick)>xl</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Shape</label>
            <select size="sm" bind:value={shape} label="Shape">
              <option value="round">round</option>
              <option value="square">square</option>
            </select>
          </column>
          <column gap="2">
            <label style(knobLabel)>Tone</label>
            <row gap="2" align="center" style(swatchRow)>
              <tooltip content="rest gradient" placement="top">
                <div label="rest" on:click={clearTone} selected={!avTone() ? 'true' : null} style(swatch, swatchRest)></div>
              </tooltip>
              <tooltip content="primary" placement="top">
                <div label="primary" on:click={() => pickTone('primary')} selected={avTone() === 'primary' ? 'true' : null} style(swatch, swatchPrimary)></div>
              </tooltip>
              <tooltip content="accent" placement="top">
                <div label="accent" on:click={() => pickTone('accent')} selected={avTone() === 'accent' ? 'true' : null} style(swatch, swatchAccent)></div>
              </tooltip>
              <tooltip content="info" placement="top">
                <div label="info" on:click={() => pickTone('info')} selected={avTone() === 'info' ? 'true' : null} style(swatch, swatchInfo)></div>
              </tooltip>
              <tooltip content="success" placement="top">
                <div label="success" on:click={() => pickTone('success')} selected={avTone() === 'success' ? 'true' : null} style(swatch, swatchSuccess)></div>
              </tooltip>
              <tooltip content="warning" placement="top">
                <div label="warning" on:click={() => pickTone('warning')} selected={avTone() === 'warning' ? 'true' : null} style(swatch, swatchWarning)></div>
              </tooltip>
              <tooltip content="destructive" placement="top">
                <div label="destructive" on:click={() => pickTone('destructive')} selected={avTone() === 'destructive' ? 'true' : null} style(swatch, swatchDestructive)></div>
              </tooltip>
              <tooltip content="muted" placement="top">
                <div label="muted" on:click={() => pickTone('muted')} selected={avTone() === 'muted' ? 'true' : null} style(swatch, swatchMuted)></div>
              </tooltip>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Name</label>
            <select size="sm" bind:value={name} label="Name">
              <option value="Ada Lovelace">Ada Lovelace</option>
              <option value="Grace Hopper">Grace Hopper</option>
              <option value="Alan Turing">Alan Turing</option>
            </select>
            <p style(mutedXs)>Fallback initials from name when initials empty.</p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Initials</label>
            <select size="sm" bind:value={initials} label="Initials">
              <option value="">(from name)</option>
              <option value="AL">AL</option>
              <option value="GH">GH</option>
              <option value="SK">SK</option>
            </select>
          </column>
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <column gap="1" style(faceRow)>
              <p style(mutedXs)><code>component.avatar.bg</code> · <code>fg</code></p>
              <p style(mutedXs)><code>src</code> / <code>icon-slug</code> demos in examples (docsSkip knobs).</p>
            </column>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Name + initials</h3>
        <p style(exampleLead)>Broken src falls back to initials via use:mediaFallback.</p>
        <row gap="2" align="center">
          <avatar name="Ada Lovelace"></avatar>
          <avatar initials="GH" tone="primary"></avatar>
        </row>
        <code-block code={avExName()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Square large</h3>
        <p style(exampleLead)><code>shape="square"</code> · <code>size="lg"</code>.</p>
        <avatar name="Alan Turing" size="lg" shape="square"></avatar>
        <code-block code={avExShape()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Size ladder</h3>
        <p style(exampleLead)>xs · sm · md · lg · xl — registry only, no product invent.</p>
        <row gap="2" align="center">
          <avatar size="xs" initials="XS"></avatar>
          <avatar size="sm" initials="SM"></avatar>
          <avatar size="md" initials="MD"></avatar>
          <avatar size="lg" initials="LG"></avatar>
          <avatar size="xl" initials="XL"></avatar>
        </row>
        <code-block code={avExSizes()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
