<!--
  <doc-prose> — /components/prose
  Shared chrome: split + snippet copy + usage Code|Live|Split.
  No size / variant / tone host props — wrap plain tags.
-->

<scope>
  style page = { size({ w: 100%, max-w: breakpoint-desktop }).margin({ x: auto }) }
  style specimen = { size({ w: panel-max-w-md, max-w: 100% }) }
  style mutedXs = { fg(muted-foreground).type(body-sm) }
  style knobLabel = { fg(muted-foreground).type(label) }
  style exampleBlock = { size({ w: 100% }).pad({ t: 2, b: 4 }) }
  style sectionLabel = { fg(muted-foreground).type(label) }
</scope>

<controller>
import type { Scope } from './doc-prose.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  // Docs-only content switch — not a host prop
  const sample = signal('article')
  const entry = DETAILS.prose
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    switch (sample()) {
      case 'quote':
        return `<prose>\n  <blockquote>Write plain tags — the type scale is the product.</blockquote>\n</prose>`
      case 'list':
        return `<prose>\n  <h3>Checklist</h3>\n  <ul>\n    <li>Wrap markdown blobs in prose</li>\n    <li>Theme nested type via the host</li>\n  </ul>\n</prose>`
      default:
        return `<prose>\n  <h2>Prose scope</h2>\n  <p>Body with <strong>emphasis</strong>, <em>italics</em>, and <code>inline()</code>.</p>\n</prose>`
    }
  })

  const proseExWrap = () =>
    `<prose>\n  <h2>Release notes</h2>\n  <p>Body with <strong>emphasis</strong> and <code>inline()</code>.</p>\n  <ul><li>One</li><li>Two</li></ul>\n</prose>`
  const proseExQuote = () =>
    `<prose>\n  <blockquote>A quotation carries its own rule.</blockquote>\n</prose>`
  const proseExBody = () =>
    `<prose>\n  <p>Wrap long copy — headings, body, lists, and code.</p>\n</prose>`

  return {
    sample,
    specimenCode,
    heading,
    blurb,
    props,
    usageCode,
    proseExWrap,
    proseExQuote,
    proseExBody,
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
      <if {sample() === 'quote'}>
        <prose>
          <blockquote>Write plain tags — the type scale is the product.</blockquote>
        </prose>
      {else =>
        <if {sample() === 'list'}>
          <prose>
            <h3>Checklist</h3>
            <ul>
              <li>Wrap markdown blobs in <code>prose</code></li>
              <li>Nested headings, lists, and code pick up the type scale</li>
            </ul>
          </prose>
        {else =>
          <prose>
            <h2>Prose scope</h2>
            <p>
              Body type with <strong>emphasis</strong>, <em>italics</em>, and
              <code>inline()</code> — nested tags pick up the framework type map.
            </p>
          </prose>}</if>}</if>
    </column>

    <column slot="knobs" gap="5" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Demo sample</label>
        <select size="sm" bind:value={sample} label="Demo sample">
          <option value="article">article</option>
          <option value="quote">quote</option>
          <option value="list">list</option>
        </select>
        <p style(mutedXs)>
          Docs-only content demo — not a host prop. No size / variant / tone axis.
        </p>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="stretch" style(specimen)>
      <prose>
        <h2>Release notes</h2>
        <p>Body with <strong>emphasis</strong> and <code>inline()</code>.</p>
      </prose>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <doc-example
        title="Wrap long copy"
        lead="Drop markdown-shaped markup inside prose — headings, body, lists."
        code={proseExWrap()}
      >
        <prose style(specimen)>
          <h2>Release notes</h2>
          <p>Body with <strong>emphasis</strong> and <code>inline()</code>.</p>
          <ul>
            <li>Surface scaffold B4</li>
            <li>Display / feedback peel</li>
          </ul>
        </prose>
      </doc-example>

      <doc-example
        title="Blockquote"
        lead="The left rule is the quote mark — no extra props."
        code={proseExQuote()}
      >
        <prose style(specimen)>
          <blockquote>A quotation carries its own rule.</blockquote>
        </prose>
      </doc-example>

      <doc-example
        title="Body only"
        lead="A single paragraph is enough when you just need readable body type."
        code={proseExBody()}
      >
        <prose style(specimen)>
          <p>Wrap long copy — headings, body, lists, and code.</p>
        </prose>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
