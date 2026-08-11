<!--
  <doc-code-block> — first-class docs host for /components/code-block.
  Specimen left, knobs right — code content + lang authoring label.
-->

<controller>
import type { Scope } from './doc-code-block.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const lang = signal('sail')
  const code = signal('const n = signal(0)\nconst double = derived(() => n() * 2)')
  const codeMeta = derived(() => {
    const lines = code().split('\n').length
    return `lang=${lang()} · ${lines} line${lines === 1 ? '' : 's'} · component.code-block.*`
  })
  const entry = DETAILS['code-block']
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const cbExBasic = () =>
    `<code-block\n  code={'const n = signal(0)'}\n  lang="sail"\n></code-block>`
  const cbExMarkup = () =>
    `<code-block\n  code={'<button variant="solid">Save</button>'}\n  lang="sail"\n></code-block>`
  const cbExRest = () =>
    `<code-block code={'// rest → component.code-block.bg / fg'}></code-block>\n<!-- lang defaults to sail; highlighter classifies markup vs code by shape -->`
  const setLang = (l: string): void => {
    lang.set(l)
    if (l === 'ts') {
      code.set('const n = signal(0)\nconst double = derived(() => n() * 2)')
    } else if (l === 'css') {
      code.set('.chip {\n  border-radius: var(--radius-full);\n}')
    } else {
      code.set('<button variant="solid" tone="primary">Save</button>')
    }
  }
  return {
    lang,
    code,
    codeMeta,
    setLang,
    heading,
    blurb,
    props,
    usageCode,
    cbExBasic,
    cbExMarkup,
    cbExRest,
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
  style fullW = { box: w(full) }
  style knobPanel = {
    box: w(panel-max-w-sm).shrink(0).pad(5).minH(64),
    paint: borderLWidth(1).borderL(border-subtle),
  }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style exampleBlock = { box: w(full).padY(2) }
  style exampleH = { text: fontSize(md).fontWeight(semibold) }
  style exampleLead = { paint: fg(muted-foreground), text: fontSize(sm) }
  style sectionLabel = { paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) }
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
            <code-block code={code()} lang={lang()} style(fullW)></code-block>
            <p style(monoXs)>{codeMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Lang</label>
            <select size="sm" bind:value={lang} label="Lang" on:change={() => setLang(lang())}>
              <option value="sail">sail</option>
              <option value="ts">ts</option>
              <option value="css">css</option>
            </select>
            <p style(mutedXs)>
              Authoring label (default <code>sail</code>). Highlighter classifies by content shape — not line numbers or copy chrome.
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
            ></textarea>
            <p style(mutedXs)>
              Pass source as the <code>code</code> string prop — not children.
              Paint on the inner <code>pre</code> from <code>component.code-block.*</code>.
            </p>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <column gap="2">
        <h3 style(exampleH)>Signal snippet</h3>
        <p style(exampleLead)>
          Default <code>lang="sail"</code> — host runs <code>use:syntaxHighlight</code>.
        </p>
        <code-block
          code={'const n = signal(0)\nconst double = derived(() => n() * 2)'}
          lang="sail"
          style(stageCol)
        ></code-block>
        <code-block code={cbExBasic()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Markup shape</h3>
        <p style(exampleLead)>
          Markup-shaped source gets tag/attr token skins automatically.
        </p>
        <code-block
          code={'<button variant="solid" tone="primary">Save</button>'}
          lang="sail"
          style(stageCol)
        ></code-block>
        <code-block code={cbExMarkup()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Rest tokens (themeable)</h3>
        <p style(exampleLead)>
          <code>component.code-block.bg</code> · <code>fg</code> plus global border/radius.
        </p>
        <code-block
          code={'// theme component.code-block.* leaves'}
          style(stageCol)
        ></code-block>
        <code-block code={cbExRest()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
