<!--
  <doc-error-state> — /components/error-state
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style specimen = { box: w(full).maxW(panel-max-w-md).pad(2).overflow(visible) }
  style fullW = { box: w(full) }
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
import type { Scope } from './doc-error-state.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const ER_SIZES = ['sm', 'md', 'lg'] as const
  const sizeIdx = signal(1)
  const variant = signal('default')
  const errCode = signal('ERR_TIMEOUT')

  const erSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), ER_SIZES.length - 1)
    return ER_SIZES[i] ?? 'md'
  })

  const entry = DETAILS['error-state']
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = []
    if (errCode()) a.push(`code="${errCode()}"`)
    if (variant() !== 'default') a.push(`variant="${variant()}"`)
    if (erSize() !== 'md') a.push(`size="${erSize()}"`)
    const open = a.length ? `<error-state ${a.join(' ')}>` : `<error-state>`
    return `${open}\n  <strong slot="title">Could not load your data</strong>\n  The request failed after 3 retries.\n  <button slot="action" variant="solid" tone="destructive" size="sm">Retry</button>\n</error-state>`
  })

  const erExDefault = () =>
    `<error-state code="ERR_TIMEOUT">\n  <strong slot="title">Could not load your data</strong>\n  The request failed after 3 retries.\n  <button slot="action" variant="solid" tone="destructive" size="sm">Retry</button>\n</error-state>`
  const erExInline = () =>
    `<error-state variant="inline">\n  <strong slot="title">Partial failure</strong>\n  One pane could not refresh.\n  <button slot="action" variant="outline" size="sm">Retry</button>\n</error-state>`
  const erExDefaultColors = () =>
    `<error-state>\n  <strong slot="title">Something went wrong</strong>\n  Default theme colors for the card chrome.\n</error-state>`

  return {
    sizeIdx,
    variant,
    errCode,
    erSize,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    erExDefault,
    erExInline,
    erExDefaultColors,
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
      <error-state code={errCode()} variant={variant()} size={erSize()} style(fullW)>
        <strong slot="title">Could not load your data</strong>
        The request failed after 3 retries. Check your connection and try again.
        <span slot="details">at fetchData (api.ts:42)
  at load (dashboard.ts:19)
  Error: gateway timeout</span>
        <button slot="action" type="button" variant="solid" tone="destructive" size="sm">
          Retry
        </button>
      </error-state>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Variant</label>
        <select size="sm" bind:value={variant} label="Variant">
          <option value="default">default</option>
          <option value="inline">inline</option>
        </select>
      </column>
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <label style(knobLabel)>Size</label>
          <span style(sizeValue)>{erSize()}</span>
        </row>
        <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size" ticks="sm,md,lg"></slider>
      </column>
      <column gap="2">
        <label style(knobLabel)>Code</label>
        <input type="text" bind:value={errCode} size="sm" placeholder="ERR_…" style(fullW) />
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="stretch" style(specimen)>
      <error-state code="ERR_TIMEOUT" style(fullW)>
        <strong slot="title">Could not load your data</strong>
        The request failed after 3 retries.
        <button slot="action" type="button" variant="solid" tone="destructive" size="sm">
          Retry
        </button>
      </error-state>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Default recovery"
        lead="Alert role, optional code chip, details gate, and a focused action."
        code={erExDefault()}
      >
        <error-state code="ERR_TIMEOUT" style(specimen)>
          <strong slot="title">Could not load your data</strong>
          The request failed after 3 retries.
          <button slot="action" type="button" variant="solid" tone="destructive" size="sm">
            Retry
          </button>
        </error-state>
      </doc-example>
      <doc-example
        title="Inline"
        lead="Start-aligned layout for partial failures inside a pane."
        code={erExInline()}
      >
        <error-state variant="inline" style(specimen)>
          <strong slot="title">Partial failure</strong>
          One pane could not refresh.
          <button slot="action" type="button" variant="outline" size="sm">Retry</button>
        </error-state>
      </doc-example>
      <doc-example
        title="Default theme colors"
        lead="Omit custom chrome — default theme colors for the card surface."
        code={erExDefaultColors()}
      >
        <error-state style(specimen)>
          <strong slot="title">Something went wrong</strong>
          Default theme colors for the card chrome.
        </error-state>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
