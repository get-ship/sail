<!--
  <doc-empty-state> — /components/empty-state
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { size({ w: 100%, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style specimen = { size({ w: 100%, max-w: panel-max-w-md }).pad(2).overflow({ all: visible }) }
  style fullW = { size({ w: 100% }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style sizeValue = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style tickRow = { size({ w: 100% }).pad({ x: 1_5, t: 1 }) }
  style tick = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style swatch = { size(7).shrink(0).radius(full).cursor(pointer).border({ color: transparent, width: 2 }).elevation(xs), when: { selected: border({ color: foreground }).elevation(sm), hover: filter({ opacity: 90 }) } }
  style swatchPrimary = { bg(primary) }
  style swatchSuccess = { bg(success) }
  style swatchWarning = { bg(warning) }
  style swatchDanger = { bg(danger) }
  style swatchAccent = { bg(accent) }
  style swatchMuted = { bg(muted).border({ color: border }) }
  style swatchInfo = { bg(info) }
  style exampleBlock = { size({ w: 100% }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
</scope>

<controller>
import type { Scope } from './doc-empty-state.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const ES_SIZES = ['sm', 'md', 'lg'] as const
  const sizeIdx = signal(1)
  const variant = signal('default')
  const esTone = signal('muted')

  const pickEsTone = (t: string): void => esTone.set(t)

  const esSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), ES_SIZES.length - 1)
    return ES_SIZES[i] ?? 'md'
  })
  const esTitle = derived(() => {
    switch (variant()) {
      case 'connecting':
        return 'Connecting…'
      case 'placeholder':
        return 'Drop a file here'
      case 'inline':
        return 'Nothing in this view'
      default:
        return 'No results'
    }
  })
  const esBody = derived(() => {
    switch (variant()) {
      case 'connecting':
        return 'Establishing a secure session.'
      case 'placeholder':
        return 'Or click to browse your disk.'
      case 'inline':
        return 'Adjust filters to see more results.'
      default:
        return 'Try a different search, or clear a filter.'
    }
  })
  const esIcon = derived(() => (variant() === 'connecting' ? 'refresh' : 'inbox'))

  const entry = DETAILS['empty-state']
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = []
    if (variant() !== 'default') a.push(`variant="${variant()}"`)
    if (esSize() !== 'md') a.push(`size="${esSize()}"`)
    a.push(`tone="${esTone()}"`)
    const open = `<empty-state ${a.join(' ')}>`
    const action =
      variant() === 'connecting'
        ? ''
        : `\n  <button slot="action" variant="outline" size="sm">Clear filters</button>`
    return `${open}\n  <icon slot="icon" name="${esIcon()}"></icon>\n  <strong slot="title">${esTitle()}</strong>\n  ${esBody()}${action}\n</empty-state>`
  })

  const esExDefault = () =>
    `<empty-state>\n  <icon slot="icon" name="inbox"></icon>\n  <strong slot="title">No results</strong>\n  Try a different search.\n  <button slot="action" variant="outline" size="sm">Clear filters</button>\n</empty-state>`
  const esExConnecting = () =>
    `<empty-state variant="connecting">\n  <icon slot="icon" name="refresh"></icon>\n  <strong slot="title">Connecting…</strong>\n  Establishing a secure session.\n</empty-state>`
  const esExDefaultColors = () =>
    `<empty-state tone="muted">\n  <icon slot="icon" name="inbox"></icon>\n  <strong slot="title">Empty</strong>\n  tone defaults to muted and paints the icon.\n</empty-state>`

  return {
    sizeIdx,
    variant,
    esTone,
    pickEsTone,
    esSize,
    esTitle,
    esBody,
    esIcon,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    esExDefault,
    esExConnecting,
    esExDefaultColors,
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
      <empty-state variant={variant()} size={esSize()} tone={esTone()} style(fullW)>
        <icon
          slot="icon"
          name={esIcon()}
        ></icon>
        <strong slot="title">{esTitle()}</strong>
        {esBody()}
        <if {variant() !== 'connecting'}>
          <button slot="action" variant="outline" size="sm">Clear filters</button>
        </if>
      </empty-state>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Variant</label>
        <select size="sm" bind:value={variant} label="Variant">
          <option value="default">default</option>
          <option value="inline">inline</option>
          <option value="placeholder">placeholder</option>
          <option value="connecting">connecting</option>
        </select>
      </column>
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <label style(knobLabel)>Size</label>
          <span style(sizeValue)>{esSize()}</span>
        </row>
        <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size" ticks="sm,md,lg"></slider>
      </column>
      <column gap="2">
        <label style(knobLabel)>Tone</label>
        <row gap="2" align="center">
          <div
            title="primary"
            selected={esTone() === 'primary' ? 'true' : null}
            on:click={() => pickEsTone('primary')}
            style(swatch, swatchPrimary)
          ></div>
          <div
            title="success"
            selected={esTone() === 'success' ? 'true' : null}
            on:click={() => pickEsTone('success')}
            style(swatch, swatchSuccess)
          ></div>
          <div
            title="warning"
            selected={esTone() === 'warning' ? 'true' : null}
            on:click={() => pickEsTone('warning')}
            style(swatch, swatchWarning)
          ></div>
          <div
            title="danger"
            selected={esTone() === 'danger' ? 'true' : null}
            on:click={() => pickEsTone('danger')}
            style(swatch, swatchDanger)
          ></div>
          <div
            title="accent"
            selected={esTone() === 'accent' ? 'true' : null}
            on:click={() => pickEsTone('accent')}
            style(swatch, swatchAccent)
          ></div>
          <div
            title="muted"
            selected={esTone() === 'muted' ? 'true' : null}
            on:click={() => pickEsTone('muted')}
            style(swatch, swatchMuted)
          ></div>
          <div
            title="info"
            selected={esTone() === 'info' ? 'true' : null}
            on:click={() => pickEsTone('info')}
            style(swatch, swatchInfo)
          ></div>
        </row>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="stretch" style(specimen)>
      <empty-state style(fullW)>
        <icon slot="icon" name="inbox"></icon>
        <strong slot="title">No results</strong>
        Try a different search, or clear a filter.
        <button slot="action" variant="outline" size="sm">Clear filters</button>
      </empty-state>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Default card"
        lead="Icon + title + body + action slots. Empty slots collapse."
        code={esExDefault()}
      >
        <empty-state style(specimen)>
          <icon slot="icon" name="inbox"></icon>
          <strong slot="title">No results</strong>
          Try a different search, or clear a filter.
          <button slot="action" variant="outline" size="sm">Clear filters</button>
        </empty-state>
      </doc-example>
      <doc-example
        title="Connecting"
        lead='variant="connecting" is a polite live status while a session starts.'
        code={esExConnecting()}
      >
        <empty-state variant="connecting" style(specimen)>
          <icon slot="icon" name="refresh"></icon>
          <strong slot="title">Connecting…</strong>
          Establishing a secure session.
        </empty-state>
      </doc-example>
      <doc-example
        title="Default tone"
        lead="tone defaults to muted and paints the icon slot."
        code={esExDefaultColors()}
      >
        <empty-state tone="muted" style(specimen)>
          <icon slot="icon" name="inbox"></icon>
          <strong slot="title">Empty</strong>
          tone defaults to muted and paints the icon.
        </empty-state>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
