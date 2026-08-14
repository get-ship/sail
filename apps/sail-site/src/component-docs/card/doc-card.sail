<!--
  <doc-card> — /components/card
  Shared chrome: split + snippet copy + usage Code|Live|Split.
  Variants: default | outline only.
-->

<scope>
  style page = { size({ w: full, max-w: breakpoint-desktop }).margin({ x: auto }) }
  style specimen = { size({ w: panel-max-w-md, max-w: full }) }
  style mutedXs = { fg(muted-foreground).type({ size: xs }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style sizeValue = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style tickRow = { size({ w: full }).pad({ x: 1_5, t: 1 }) }
  style tick = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style swatch = { size(7).shrink(0).radius(full).cursor(pointer).border({ color: transparent, width: 2 }).elevation(xs), when: { selected: border({ color: foreground }).elevation(sm), hover: filter({ opacity: 90 }) } }
  style swatchRest = { bg(surface-1).border({ color: border }) }
  style swatchPrimary = { bg(primary) }
  style swatchInfo = { bg(info) }
  style swatchSuccess = { bg(success) }
  style swatchWarning = { bg(warning) }
  style swatchDestructive = { bg(destructive) }
  style swatchAccent = { bg(accent) }
  style swatchMuted = { bg(muted).border({ color: border }) }
  style exampleBlock = { size({ w: full }).pad({ t: 2, b: 4 }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
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
    `<card>\n  <strong slot="header">Harbor Signals</strong>\n  <p>4 tracks · last played today</p>\n  <row slot="footer" gap="2">\n    <button size="sm" variant="ghost">Skip</button>\n    <button size="sm" variant="solid" tone="primary">Play</button>\n  </row>\n</card>`
  const cardExInteractive = () =>
    `<card interactive>\n  <strong>Open project</strong>\n  <p>Hover lifts the surface. No tone required.</p>\n</card>`
  const cardExOutline = () =>
    `<card variant="outline" status-rail>\n  <strong slot="header">Deploy</strong>\n  <p>Pipeline passed on main.</p>\n</card>`

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
        <slider bind:value={padIdx} min="0" max="3" step="1" size="sm" label="Padding" ticks="none,sm,md,lg"></slider>
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
        title="Header, body, footer"
        lead="Slots give a real card. Default surface is the theme card — no extra tone."
        code={cardExDefault()}
      >
        <card style(specimen)>
          <strong slot="header">Harbor Signals</strong>
          <p style(mutedXs)>4 tracks · last played today</p>
          <row slot="footer" gap="2">
            <button size="sm" variant="ghost">Skip</button>
            <button size="sm" variant="solid" tone="primary">Play</button>
          </row>
        </card>
      </doc-example>

      <doc-example
        title="Interactive"
        lead="interactive adds hover lift. Leave tone off unless you want a tinted border."
        code={cardExInteractive()}
      >
        <card interactive style(specimen)>
          <strong>Open project</strong>
          <p style(mutedXs)>Hover lifts the surface. No tone required.</p>
        </card>
      </doc-example>

      <doc-example
        title="Outline + status rail"
        lead="outline for a lighter shell; status-rail for a leading stripe."
        code={cardExOutline()}
      >
        <card variant="outline" status-rail style(specimen)>
          <strong slot="header">Deploy</strong>
          <p style(mutedXs)>Pipeline passed on main.</p>
        </card>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
