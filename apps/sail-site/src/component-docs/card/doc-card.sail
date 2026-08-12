<!--
  <doc-card> — /components/card
  Shared chrome: split + snippet copy + usage Code|Live|Split.
  Variants: default | outline only.
-->

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style specimen = { box: w(panel-max-w-md).maxW(full) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style tickRow = { box: w(full).padT(1).padX(1_5) }
  style tick = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style swatch = {
    box: size(7).shrink(0),
    paint: radius(full).cursor(pointer).borderWidth(2).border(transparent).shadow(xs),
    when: {
      selected: { paint: border(foreground).shadow(sm) },
      hover: { paint: opacity(90) },
    },
  }
  style swatchRest = { paint: bg(surface-1).border(border) }
  style swatchPrimary = { paint: bg(primary) }
  style swatchInfo = { paint: bg(info) }
  style swatchSuccess = { paint: bg(success) }
  style swatchWarning = { paint: bg(warning) }
  style swatchDestructive = { paint: bg(destructive) }
  style swatchAccent = { paint: bg(accent) }
  style swatchMuted = { paint: bg(muted).border(border) }
  style exampleBlock = { box: w(full).padT(2).padB(4) }
  style sectionLabel = { paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) }
</scope>

<controller>
import type { Scope } from './doc-card.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const PAD_SIZES = ['none', 'sm', 'md', 'lg'] as const
  const variant = signal('default')
  const padIdx = signal(2)
  const cardTone = signal('')
  const interactive = signal(false)
  const accentBar = signal('off')
  const statusRail = signal(false)
  const fill = signal(false)
  const pickTone = (t: string): void => cardTone.set(t)
  const clearTone = (): void => cardTone.set('')

  const padding = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(padIdx()) || 0)), PAD_SIZES.length - 1)
    return PAD_SIZES[i] ?? 'md'
  })

  const entry = DETAILS.card
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = []
    if (variant() !== 'default') a.push(`variant="${variant()}"`)
    if (padding() !== 'md') a.push(`padding="${padding()}"`)
    if (cardTone()) a.push(`tone="${cardTone()}"`)
    if (interactive()) a.push('interactive')
    if (accentBar() !== 'off') a.push(`accent-bar="${accentBar()}"`)
    if (statusRail()) a.push('status-rail')
    if (fill()) a.push('fill')
    const open = a.length ? `<card ${a.join(' ')}>` : `<card>`
    return `${open}\n  <strong>Harbor Signals</strong>\n  <p>4 tracks · last played today</p>\n</card>`
  })

  const cardExDefault = () =>
    `<card>\n  <strong>Harbor Signals</strong>\n  <p>4 tracks · last played today</p>\n</card>`
  const cardExInteractive = () =>
    `<card interactive accent-bar="static" tone="primary">\n  <strong>Open project</strong>\n</card>`
  const cardExOutline = () =>
    `<card variant="outline" padding="lg" status-rail>\n  <strong>Outline shell</strong>\n</card>`

  return {
    variant,
    padIdx,
    padding,
    cardTone,
    interactive,
    accentBar,
    statusRail,
    fill,
    pickTone,
    clearTone,
    specimenCode,
    heading,
    blurb,
    props,
    usageCode,
    cardExDefault,
    cardExInteractive,
    cardExOutline,
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
      <if {cardTone()}>
        <card
          variant={variant()}
          padding={padding()}
          tone={cardTone()}
          interactive={interactive() ? 'true' : null}
          accent-bar={accentBar()}
          status-rail={statusRail() ? 'true' : null}
          fill={fill() ? 'true' : null}
          style(specimen)
        >
          <strong>Harbor Signals</strong>
          <p style(mutedXs)>4 tracks · last played today</p>
        </card>
      {else =>
        <card
          variant={variant()}
          padding={padding()}
          interactive={interactive() ? 'true' : null}
          accent-bar={accentBar()}
          status-rail={statusRail() ? 'true' : null}
          fill={fill() ? 'true' : null}
          style(specimen)
        >
          <strong>Harbor Signals</strong>
          <p style(mutedXs)>4 tracks · last played today</p>
        </card>}</if>
    </column>

    <column slot="knobs" gap="5" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Variant</label>
        <select size="sm" bind:value={variant} label="Variant">
          <option value="default">default</option>
          <option value="outline">outline</option>
        </select>
      </column>
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <label style(knobLabel)>Padding</label>
          <span style(sizeValue)>{padding()}</span>
        </row>
        <slider bind:value={padIdx} min="0" max="3" step="1" size="sm" label="Padding"></slider>
        <row gap="0" align="center" justify="between" style(tickRow)>
          <span style(tick)>none</span>
          <span style(tick)>sm</span>
          <span style(tick)>md</span>
          <span style(tick)>lg</span>
        </row>
      </column>
      <column gap="2">
        <label style(knobLabel)>Tone</label>
        <row gap="2" align="center">
          <div
            title="default (theme)"
            selected={!cardTone() ? 'true' : null}
            on:click={clearTone}
            style(swatch, swatchRest)
          ></div>
          <div
            title="primary"
            selected={cardTone() === 'primary' ? 'true' : null}
            on:click={() => pickTone('primary')}
            style(swatch, swatchPrimary)
          ></div>
          <div
            title="info"
            selected={cardTone() === 'info' ? 'true' : null}
            on:click={() => pickTone('info')}
            style(swatch, swatchInfo)
          ></div>
          <div
            title="success"
            selected={cardTone() === 'success' ? 'true' : null}
            on:click={() => pickTone('success')}
            style(swatch, swatchSuccess)
          ></div>
          <div
            title="warning"
            selected={cardTone() === 'warning' ? 'true' : null}
            on:click={() => pickTone('warning')}
            style(swatch, swatchWarning)
          ></div>
          <div
            title="destructive"
            selected={cardTone() === 'destructive' ? 'true' : null}
            on:click={() => pickTone('destructive')}
            style(swatch, swatchDestructive)
          ></div>
          <div
            title="accent"
            selected={cardTone() === 'accent' ? 'true' : null}
            on:click={() => pickTone('accent')}
            style(swatch, swatchAccent)
          ></div>
          <div
            title="muted"
            selected={cardTone() === 'muted' ? 'true' : null}
            on:click={() => pickTone('muted')}
            style(swatch, swatchMuted)
          ></div>
        </row>
      </column>
      <column gap="2">
        <label style(knobLabel)>Accent bar</label>
        <select size="sm" bind:value={accentBar} label="Accent bar">
          <option value="off">off</option>
          <option value="hover">hover</option>
          <option value="static">static</option>
        </select>
      </column>
      <row gap="4" align="center">
        <label><checkbox bind:checked={interactive} size="sm" /> Interactive</label>
        <label><checkbox bind:checked={statusRail} size="sm" /> Status rail</label>
        <label><checkbox bind:checked={fill} size="sm" /> Fill</label>
      </row>
    </column>

    <column slot="usage-preview" gap="3" align="start" style(specimen)>
      <card style(specimen)>
        <strong>Harbor Signals</strong>
        <p style(mutedXs)>4 tracks · last played today</p>
      </card>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <doc-example
        title="Default surface"
        lead="Theme fill when no tone is set."
        code={cardExDefault()}
      >
        <card style(specimen)>
          <strong>Harbor Signals</strong>
          <p style(mutedXs)>4 tracks · last played today</p>
        </card>
      </doc-example>

      <doc-example
        title="Interactive + accent"
        lead='interactive hover lift; accent-bar="static" with tone.'
        code={cardExInteractive()}
      >
        <card interactive accent-bar="static" tone="primary" style(specimen)>
          <strong>Open project</strong>
          <p style(mutedXs)>Pointer + lift, one attribute.</p>
        </card>
      </doc-example>

      <doc-example
        title="Outline + status rail"
        lead='variant="outline" and status-rail for a leading rail.'
        code={cardExOutline()}
      >
        <card variant="outline" padding="lg" status-rail style(specimen)>
          <strong>Outline shell</strong>
          <p style(mutedXs)>Transparent body, leading rail.</p>
        </card>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
