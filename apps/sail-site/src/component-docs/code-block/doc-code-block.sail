<!--
  <doc-code-block> — /components/code-block
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { size({ w: 100%, max-w: breakpoint-desktop }).margin({ x: auto }) }
  style specimen = { size({ w: panel-max-w-md, max-w: 100% }) }
  style fullW = { size({ w: 100% }) }
  style mutedXs = { fg(muted-foreground).type(body-sm) }
  style knobLabel = { fg(muted-foreground).type(label) }
  style exampleBlock = { size({ w: 100% }).pad({ t: 2, b: 4 }) }
  style sectionLabel = { fg(muted-foreground).type(label) }
</scope>

<controller>
import type { Scope } from './doc-code-block.sail'
import { DETAILS } from '../../compositions/component-details'
import {
  CB_MARKUP,
  CB_PLAIN,
  CB_SIGNAL,
  cbExBasic,
  cbExMarkup,
  cbExPlain,
  codeBlockSnippet,
  sampleForLang,
} from '../../compositions/code-block-snippets'

export default controller((_ctx: Scope) => {
  const lang = signal('sail')
  const code = signal(CB_SIGNAL)

  const entry = DETAILS['code-block']
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => codeBlockSnippet(code(), lang()))
  const setLang = (l: string): void => {
    lang.set(l)
    code.set(sampleForLang(l))
  }

  return {
    lang,
    code,
    specimenCode,
    setLang,
    heading,
    blurb,
    props,
    usageCode,
    cbExBasic,
    cbExMarkup,
    cbExPlain,
    CB_SIGNAL,
    CB_MARKUP,
    CB_PLAIN,
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
      <code-block code={code()} lang={lang()} style(fullW)></code-block>
    </column>

    <column slot="knobs" gap="5" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Lang</label>
        <select size="sm" bind:value={lang} label="Lang" on:change={() => setLang(lang())}>
          <option value="sail">sail</option>
          <option value="ts">ts</option>
          <option value="css">css</option>
        </select>
        <p style(mutedXs)>
          Authoring label (default sail). The highlighter classifies by content shape.
        </p>
      </column>
      <column gap="2">
        <label style(knobLabel)>Code</label>
        <textarea
          bind:value={code}
          size="sm"
          rows="6"
          placeholder="source text"
          style(fullW)
        />
        <p style(mutedXs)>Pass source as the code string prop — not children.</p>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="stretch" style(specimen)>
      <code-block
        code={'const n = signal(0)\nconst double = derived(() => n() * 2)'}
        lang="sail"
        style(fullW)
      ></code-block>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <doc-example
        title="Signal snippet"
        lead='Default lang="sail" with syntax highlighting.'
        code={cbExBasic()}
      >
        <code-block
          code={'const n = signal(0)\nconst double = derived(() => n() * 2)'}
          lang="sail"
          style(specimen)
        ></code-block>
      </doc-example>

      <doc-example
        title="Markup shape"
        lead="Markup-shaped source gets tag and attribute highlighting automatically."
        code={cbExMarkup()}
      >
        <code-block
          code={CB_MARKUP}
          lang="sail"
          style(specimen)
        ></code-block>
      </doc-example>

      <doc-example
        title="Minimal"
        lead="lang defaults to sail when omitted."
        code={cbExPlain()}
      >
        <code-block
          code={'// your source here'}
          style(specimen)
        ></code-block>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
