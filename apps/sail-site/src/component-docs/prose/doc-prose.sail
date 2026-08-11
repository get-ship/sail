<!--
  <doc-prose> — first-class docs host for /components/prose.
  Zero PE axes — specimen + face copy; optional demo content switch (demoOnly).
-->

<controller>
import type { Scope } from './doc-prose.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  // demoOnly content switch — not a PE prop
  const sample = signal('article')
  const entry = DETAILS.prose
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const proseMeta = derived(() => {
    return `${sample()} · host ink component.prose.fg · type map from skins`
  })
  const proseExWrap = () =>
    `<prose>\n  <h2>Release notes</h2>\n  <p>Body with <strong>emphasis</strong> and <code>inline()</code>.</p>\n  <ul><li>One</li><li>Two</li></ul>\n</prose>`
  const proseExQuote = () =>
    `<prose>\n  <blockquote>A quotation carries its own rule.</blockquote>\n</prose>`
  const proseExRest = () =>
    `<prose>\n  <p>Theme <code>component.prose.fg</code> — not a Title API.</p>\n</prose>`
  return {
    sample,
    proseMeta,
    heading,
    blurb,
    props,
    usageCode,
    proseExWrap,
    proseExQuote,
    proseExRest,
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
  style exampleBlock = { box: w(full).padY(2) }
  style exampleH = { text: fontSize(md).fontWeight(semibold) }
  style exampleLead = { paint: fg(muted-foreground), text: fontSize(sm) }
  style sectionLabel = { paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) }
  style faceRow = { box: w(full) }
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
          <column gap="3" align="stretch" style(stageCol)>
            <if {sample() === 'quote'}>
              <prose>
                <blockquote>Write plain tags — the type scale is the product.</blockquote>
                <p>Host ink from <code>component.prose.fg</code>.</p>
              </prose>
            {else =>
              <if {sample() === 'list'}>
                <prose>
                  <h3>Checklist</h3>
                  <ul>
                    <li>Wrap markdown blobs in <code>prose</code></li>
                    <li>Theme the leaf, not a Title API</li>
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
            <p style(monoXs)>{proseMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Demo sample</label>
            <select size="sm" bind:value={sample} label="Demo sample">
              <option value="article">article</option>
              <option value="quote">quote</option>
              <option value="list">list</option>
            </select>
            <p style(mutedXs)>
              Docs-only content demo — not a PE prop. No size / variant / tone axis.
            </p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <column gap="1" style(faceRow)>
              <p style(mutedXs)><code>component.prose.fg</code> — host ink</p>
            </column>
            <p style(mutedXs)>
              Nested h1…p, lists, blockquote, and code use element skins — not PE props.
            </p>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <column gap="2">
        <h3 style(exampleH)>Wrap long copy</h3>
        <p style(exampleLead)>
          Drop markdown-shaped markup inside <code>prose</code> — headings, body, lists.
        </p>
        <prose style(stageCol)>
          <h2>Release notes</h2>
          <p>Body with <strong>emphasis</strong> and <code>inline()</code>.</p>
          <ul>
            <li>Surface scaffold B4</li>
            <li>Display / feedback peel</li>
          </ul>
        </prose>
        <code-block code={proseExWrap()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Blockquote</h3>
        <p style(exampleLead)>The left rule is the quote mark — no invent props.</p>
        <prose style(stageCol)>
          <blockquote>A quotation carries its own rule.</blockquote>
        </prose>
        <code-block code={proseExQuote()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Rest tokens (themeable)</h3>
        <p style(exampleLead)>
          Host ink is <code>component.prose.fg</code>. Type scale brand is operator taste.
        </p>
        <prose style(stageCol)>
          <p>Theme the leaf — not a Title API.</p>
        </prose>
        <code-block code={proseExRest()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
