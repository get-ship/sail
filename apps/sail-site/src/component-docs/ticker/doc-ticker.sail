<!--
  <doc-ticker> — /components/ticker
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { size({ w: 100%, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style specimen = { size({ w: 100%, max-w: panel-max-w-md }).pad(2).overflow({ all: visible }) }
  style exampleBlock = { size({ w: 100% }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style sectionLabel = { fg(muted-foreground).type(label) }
</scope>

<controller>
import type { Scope } from './doc-ticker.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const entry = DETAILS.ticker
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const tickerSnippet = () => '<ticker>\n  <p>tokens · signals · a11y · hot reload · SSR</p>\n</ticker>'
  const tkExDefault = () =>
    `<ticker>
  <p>tokens · signals · a11y · hot reload · SSR</p>
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
    heading,
    blurb,
    props,
    usageCode,
    tickerSnippet,
    tkExDefault,
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
    snippet={tickerSnippet()}
    usage-live
    usage={usageCode()}
    rows={props()}
  >
    <column slot="specimen" gap="3" align="stretch" style(specimen)>
      <ticker>
        <p>api 14ms · worker 41ms · edge 9ms · registry 220ms · queue 3</p>
      </ticker>
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
        code={tkExDefault()}
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
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>