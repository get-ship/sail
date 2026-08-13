<!--
  <doc-empty-state> — /components/empty-state
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { box: size({ w: full, max-w: breakpoint-desktop }).space({ mx: auto }).overflow({ all: visible }) }
  style specimen = { box: size({ w: full, max-w: panel-max-w-md }).space({ pad: 2 }).overflow({ all: visible }) }
  style fullW = { box: size({ w: full }) }
  style knobLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, weight: medium, tracking: wide, case: uppercase }),
}
  style sizeValue = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style tickRow = { box: size({ w: full }).space({ pad-x: 1_5, pad-t: 1 }) }
  style tick = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style swatch = {
    box: size(7).flow({ shrink: 0 }),
    paint: radius(full).cursor(pointer).border({ color: transparent, width: 2 }).elevation(xs),
    when: { selected: { paint: border({ color: foreground }).elevation(sm) }, hover: { paint: filter({ opacity: 90 }) } },
}
  style swatchPrimary = { paint: fill({ color: primary }) }
  style swatchSuccess = { paint: fill({ color: success }) }
  style swatchWarning = { paint: fill({ color: warning }) }
  style swatchDestructive = { paint: fill({ color: destructive }) }
  style swatchAccent = { paint: fill({ color: accent }) }
  style swatchMuted = { paint: fill({ color: muted }).border({ color: border }) }
  style swatchInfo = { paint: fill({ color: info }) }
  style swatchRest = { paint: fill({ color: background }).border({ color: border }) }
  style exampleBlock = { box: size({ w: full }).space({ pad-t: 2, pad-b: 4 }).overflow({ all: visible }) }
  style sectionLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, tracking: wide, case: uppercase }),
}
</scope>

<controller>
import type { Scope } from './doc-empty-state.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const ES_SIZES = ['sm', 'md', 'lg'] as const
  const sizeIdx = signal(1)
  const variant = signal('default')
  const esTone = signal('')

  const pickEsTone = (t: string): void => esTone.set(t)
  const clearEsTone = (): void => esTone.set('')

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
  const esIcon = derived(() => (variant() === 'connecting' ? 'refresh-cw' : 'inbox'))

  const entry = DETAILS['empty-state']
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = []
    if (variant() !== 'default') a.push(`variant="${variant()}"`)
    if (esSize() !== 'md') a.push(`size="${esSize()}"`)
    if (esTone()) a.push(`tone="${esTone()}"`)
    const open = a.length ? `<empty-state ${a.join(' ')}>` : `<empty-state>`
    const iconSpin = variant() === 'connecting' ? ' spin' : ''
    const action =
      variant() === 'connecting'
        ? ''
        : `\n  <button slot="action" variant="outline" size="sm">Clear filters</button>`
    return `${open}\n  <icon slot="icon" name="${esIcon()}"${iconSpin}></icon>\n  <strong slot="title">${esTitle()}</strong>\n  ${esBody()}${action}\n</empty-state>`
  })

  const esExDefault = () =>
    `<empty-state>\n  <icon slot="icon" name="inbox"></icon>\n  <strong slot="title">No results</strong>\n  Try a different search.\n  <button slot="action" variant="outline" size="sm">Clear filters</button>\n</empty-state>`
  const esExConnecting = () =>
    `<empty-state variant="connecting">\n  <icon slot="icon" name="refresh-cw" spin></icon>\n  <strong slot="title">Connecting…</strong>\n  Establishing a secure session.\n</empty-state>`
  const esExDefaultColors = () =>
    `<empty-state>\n  <strong slot="title">Empty</strong>\n  Omit tone for default icon colors.\n</empty-state>`

  return {
    sizeIdx,
    variant,
    esTone,
    pickEsTone,
    clearEsTone,
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
      <if {esTone()}>
        <empty-state variant={variant()} size={esSize()} tone={esTone()} style(fullW)>
          <icon
            slot="icon"
            name={esIcon()}
            spin={variant() === 'connecting' ? 'true' : null}
          ></icon>
          <strong slot="title">{esTitle()}</strong>
          {esBody()}
          <if {variant() !== 'connecting'}>
            <button slot="action" variant="outline" size="sm">Clear filters</button>
          </if>
        </empty-state>
      {else =>
        <empty-state variant={variant()} size={esSize()} style(fullW)>
          <icon
            slot="icon"
            name={esIcon()}
            spin={variant() === 'connecting' ? 'true' : null}
          ></icon>
          <strong slot="title">{esTitle()}</strong>
          {esBody()}
          <if {variant() !== 'connecting'}>
            <button slot="action" variant="outline" size="sm">Clear filters</button>
          </if>
        </empty-state>}</if>
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
            title="default"
            selected={!esTone() ? 'true' : null}
            on:click={clearEsTone}
            style(swatch, swatchRest)
          ></div>
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
            title="destructive"
            selected={esTone() === 'destructive' ? 'true' : null}
            on:click={() => pickEsTone('destructive')}
            style(swatch, swatchDestructive)
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
          <icon slot="icon" name="refresh-cw" spin></icon>
          <strong slot="title">Connecting…</strong>
          Establishing a secure session.
        </empty-state>
      </doc-example>
      <doc-example
        title="Default theme colors"
        lead="Omit tone for default icon colors."
        code={esExDefaultColors()}
      >
        <empty-state style(specimen)>
          <strong slot="title">Empty</strong>
          Omit tone for default icon colors.
        </empty-state>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
