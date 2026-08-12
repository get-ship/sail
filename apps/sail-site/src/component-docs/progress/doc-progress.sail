<!--
  <doc-progress> — /components/progress
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style specimen = { box: w(64).maxW(full).pad(2).overflow(visible) }
  style controlW = { box: w(full) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style tickRow = { box: w(full).padT(1).padX(1_5) }
  style tick = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style exampleBlock = { box: w(full).padT(2).padB(4).overflow(visible) }
  style sectionLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).textCase(uppercase).tracking(wide),
  }
</scope>

<controller>
import type { Scope } from './doc-progress.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const PRG_SIZES = ['sm', 'md', 'lg'] as const
  const sizeIdx = signal(1)
  const load = signal(62)
  const progIndeterminate = signal(false)

  const progSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), PRG_SIZES.length - 1)
    return PRG_SIZES[i] ?? 'md'
  })

  const entry = DETAILS.progress
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = ['max="100"']
    if (!progIndeterminate()) a.unshift(`value="${load()}"`)
    if (progSize() !== 'md') a.push(`size="${progSize()}"`)
    return `<progress ${a.join(' ')}></progress>`
  })

  const prgExDeterminate = () =>
    `<progress value={done()} max="100"></progress>\n<p>{done()}% uploaded</p>`
  const prgExSizes = () =>
    `<progress value="50" max="100" size="sm"></progress>\n<progress value="50" max="100"></progress>\n<progress value="50" max="100" size="lg"></progress>`
  const prgExIndeterminate = () =>
    `<progress max="100"></progress>\n<!-- omit value → indeterminate -->`

  return {
    sizeIdx,
    load,
    progIndeterminate,
    progSize,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    prgExDeterminate,
    prgExSizes,
    prgExIndeterminate,
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
    <column slot="specimen" gap="3" align="stretch" style(specimen)>
      <if {progIndeterminate()}>
        <progress max="100" size={progSize()} style(controlW)></progress>
      {else =>
        <progress value={load()} max="100" size={progSize()} style(controlW)></progress>}</if>
      <if {!progIndeterminate()}>
        <p style(mutedSm)>{load()}% complete</p>
      </if>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <label style(knobLabel)>Size</label>
          <span style(sizeValue)>{progSize()}</span>
        </row>
        <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size"></slider>
        <row gap="0" align="center" justify="between" style(tickRow)>
          <span style(tick)>sm</span>
          <span style(tick)>md</span>
          <span style(tick)>lg</span>
        </row>
      </column>
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <label style(knobLabel)>Value</label>
          <span style(sizeValue)>{progIndeterminate() ? '—' : `${load()}%`}</span>
        </row>
        <slider bind:value={load} min="0" max="100" size="sm" label="Value"></slider>
      </column>
      <column gap="2">
        <label style(knobLabel)>Indeterminate</label>
        <row gap="2" align="center">
          <switch bind:checked={progIndeterminate} size="sm" tone="primary"></switch>
          <span style(sizeValue)>{progIndeterminate() ? 'no value attr' : 'determinate'}</span>
        </row>
      </column>
    </column>

    <column slot="usage-preview" gap="3" style(specimen)>
      <progress value={load()} max="100" style(controlW)></progress>
      <p style(mutedSm)>{load()}% complete</p>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Determinate bar"
        lead="Bind value and max — the platform announces completion."
        code={prgExDeterminate()}
      >
        <column gap="2" style(specimen)>
          <progress value={load()} max="100" style(controlW)></progress>
          <p style(mutedSm)>{load()}% complete</p>
        </column>
      </doc-example>
      <doc-example title="Sizes" lead="Default is md." code={prgExSizes()}>
        <column gap="3" style(specimen)>
          <progress value="50" max="100" size="sm" style(controlW)></progress>
          <progress value="50" max="100" style(controlW)></progress>
          <progress value="50" max="100" size="lg" style(controlW)></progress>
        </column>
      </doc-example>
      <doc-example
        title="Indeterminate"
        lead="Leave value off — no separate indeterminate prop."
        code={prgExIndeterminate()}
      >
        <column gap="2" style(specimen)>
          <progress max="100" style(controlW)></progress>
        </column>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
