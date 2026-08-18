<!--
  <doc-ticker> — /components/ticker
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { size({ w: 100%, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style specimen = { size({ w: 100%, max-w: panel-max-w-md }).pad(2).overflow({ all: visible }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style sizeValue = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style mutedXs = { fg(muted-foreground).type({ size: xs }) }
  style exampleBlock = { size({ w: 100% }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
</scope>

<controller>
import type { Scope } from './doc-ticker.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const duration = signal('normal')
  const reverse = signal(false)

  const entry = DETAILS.ticker
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = []
    if (duration() !== 'normal') a.push(`duration="${duration()}"`)
    if (reverse()) a.push('reverse')
    const open = a.length ? `<ticker ${a.join(' ')}>` : '<ticker>'
    return `${open}
  <p>tokens · signals · a11y · hot reload · SSR</p>
</ticker>`
  })

  const tkExNormal = () =>
    `<ticker>
  <p>tokens · signals · a11y · hot reload · SSR</p>
</ticker>`
  const tkExReverse = () =>
    `<ticker duration="slow" reverse>
  <p>rightward marquee</p>
</ticker>`
  const tkExFast = () =>
    `<ticker duration="fast">
  <p>fast 16s loop</p>
</ticker>`
  const tkExIcons = () =>
    `<ticker>
  <row nowrap gap="3" align="center">
    <icon name="rocket" size="sm"></icon>
    <span>ship it</span>
    <icon name="zap" size="sm"></icon>
    <span>hot reload</span>
    <icon name="check" size="sm"></icon>
    <span>a11y</span>
    <icon name="sparkles" size="sm"></icon>
    <span>tokens</span>
  </row>
</ticker>`

  return {
    duration,
    reverse,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    tkExNormal,
    tkExReverse,
    tkExFast,
    tkExIcons,
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
      <if {reverse()}>
        <ticker duration={duration()} reverse>
          <p>api 14ms · worker 41ms · edge 9ms · registry 220ms · queue 3</p>
        </ticker>
      {else =>
        <ticker duration={duration()}>
          <p>api 14ms · worker 41ms · edge 9ms · registry 220ms · queue 3</p>
        </ticker>}</if>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Duration</label>
        <select size="sm" bind:value={duration} label="Duration">
          <option value="fast">fast</option>
          <option value="normal">normal</option>
          <option value="slow">slow</option>
        </select>
        <p style(mutedXs)>Loop length — fast 16s · normal 24s · slow 32s.</p>
      </column>
      <column gap="2">
        <label style(knobLabel)>Reverse</label>
        <row gap="2" align="center">
          <switch bind:checked={reverse} size="sm" label="Reverse"></switch>
          <span style(sizeValue)>{reverse() ? 'rightward' : 'leftward'}</span>
        </row>
        <p style(mutedXs)>Flips the marquee so it travels the other way.</p>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="stretch" style(specimen)>
      <ticker>
        <p>tokens · signals · a11y · hot reload · SSR</p>
      </ticker>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Default band"
        lead="A quiet strip that loops. Put text in — no extra paint required."
        code={tkExNormal()}
      >
        <ticker>
          <p>api 14ms · worker 41ms · edge 9ms · registry 220ms · queue 3</p>
        </ticker>
      </doc-example>
      <doc-example
        title="Icons in the loop"
        lead="Icons ride along with the words. Stick to the named set — rocket, zap, check, sparkles…"
        code={tkExIcons()}
      >
        <ticker>
          <row nowrap gap="3" align="center">
            <icon name="rocket" size="sm"></icon>
            <span>ship it</span>
            <icon name="zap" size="sm"></icon>
            <span>hot reload</span>
            <icon name="check" size="sm"></icon>
            <span>a11y</span>
            <icon name="sparkles" size="sm"></icon>
            <span>tokens</span>
          </row>
        </ticker>
      </doc-example>
      <doc-example
        title="Reverse + slow"
        lead="reverse walks the other way; duration=&quot;slow&quot; lengthens the loop."
        code={tkExReverse()}
      >
        <ticker duration="slow" reverse>
          <p>rightward marquee · reduced motion settles</p>
        </ticker>
      </doc-example>
      <doc-example
        title="Fast"
        lead="duration=&quot;fast&quot; — a 16s loop."
        code={tkExFast()}
      >
        <ticker duration="fast">
          <p>fast loop · paint is yours if you want it</p>
        </ticker>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
