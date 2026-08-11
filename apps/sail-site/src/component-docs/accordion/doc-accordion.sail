<!--
  <doc-accordion> — first-class docs host for /components/accordion.
  Specimen left, knobs right — open · disabled · name · multiple (onToggle live).
-->

<controller>
import type { Scope } from './doc-accordion.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const open = signal(false)
  const disabled = signal(false)
  const name = signal('')
  const multiple = signal(false)
  const lastOpen = signal('')
  const onToggle = (detail: { open?: boolean }): void => {
    const o = detail?.open
    if (typeof o === 'boolean') {
      open.set(o)
      lastOpen.set(o ? 'open' : 'closed')
    }
  }
  const accMeta = derived(() => {
    const n = name() ? ` · name "${name()}"` : ''
    const m = multiple() ? ' · multiple' : ''
    const d = disabled() ? ' · disabled' : ''
    const t = lastOpen() ? ` · last ${lastOpen()}` : ''
    return `${open() ? 'open' : 'closed'}${n}${m}${d}${t} · component.accordion.*`
  })
  const entry = DETAILS.accordion
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const accExBasic = () =>
    `<accordion>\n  <span slot="trigger">Webhook endpoints</span>\n  <p>POST /v1/hooks · signed with your workspace secret.</p>\n</accordion>`
  const accExOpen = () =>
    `<accordion open>\n  <span slot="trigger">Already open</span>\n  <p>Content visible on first paint.</p>\n</accordion>`
  const accExExclusive = () =>
    `<accordion name="faq">…</accordion>\n<accordion name="faq">…</accordion>\n<!-- shared name → exclusive (native details name) -->`
  const accExMultiple = () =>
    `<accordion multiple name="filters">…</accordion>\n<!-- multiple allows several open when not exclusiving -->`
  return {
    open, disabled, name, multiple, lastOpen, onToggle, accMeta,
    heading, blurb, props, usageCode,
    accExBasic, accExOpen, accExExclusive, accExMultiple,
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
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style exampleBlock = { box: w(full).padY(2) }
  style exampleH = { text: fontSize(md).fontWeight(semibold) }
  style exampleLead = { paint: fg(muted-foreground), text: fontSize(sm) }
  style sectionLabel = { paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) }
  style faceRow = { box: w(full) }
  style fullW = { box: w(full) }

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
            <accordion
              open={open() ? 'true' : null}
              disabled={disabled() ? 'true' : null}
              name={name() || null}
              multiple={multiple() ? 'true' : null}
              on-toggle={onToggle}
              style(fullW)
            >
              <span slot="trigger">Webhook endpoints</span>
              <p style(mutedSm)>POST /v1/hooks · signed with your workspace secret.</p>
            </accordion>
            <if {name()}>
              <accordion name={name()} multiple={multiple() ? 'true' : null} style(fullW)>
                <span slot="trigger">Sibling (same name group)</span>
                <p style(mutedSm)>Shared name = exclusive unless multiple.</p>
              </accordion>
            </if>
            <p style(monoXs)>{accMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Open</label>
            <row gap="2" align="center">
              <switch bind:checked={open} size="sm" tone="primary" label="Open"></switch>
              <span style(sizeValue)>{open() ? 'expanded' : 'collapsed'}</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Disabled</label>
            <row gap="2" align="center">
              <switch bind:checked={disabled} size="sm" tone="primary" label="Disabled"></switch>
              <span style(sizeValue)>{disabled() ? 'locked' : 'interactive'}</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Name</label>
            <select size="sm" bind:value={name} label="Name">
              <option value="">(none — independent)</option>
              <option value="faq">faq</option>
              <option value="filters">filters</option>
            </select>
            <p style(mutedXs)>Shared name → exclusive multi-item (native details name).</p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Multiple</label>
            <row gap="2" align="center">
              <switch bind:checked={multiple} size="sm" tone="primary" label="Multiple"></switch>
              <span style(sizeValue)>{multiple() ? 'several open' : 'exclusive when named'}</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <column gap="1" style(faceRow)>
              <p style(mutedXs)><code>component.accordion.bg / fg / border / hover-bg / chevron</code></p>
            </column>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Basic disclosure</h3>
        <p style(exampleLead)>Trigger slot always visible; body expands on toggle.</p>
        <accordion style(fullW)>
          <span slot="trigger">Webhook endpoints</span>
          <p style(mutedSm)>POST /v1/hooks · signed with your workspace secret.</p>
        </accordion>
        <code-block code={accExBasic()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Open by default</h3>
        <p style(exampleLead)><code>open</code> reflects initial expanded state.</p>
        <code-block code={accExOpen()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Exclusive group</h3>
        <p style(exampleLead)>Shared <code>name</code> — only one open at a time.</p>
        <code-block code={accExExclusive()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Multiple open</h3>
        <p style(exampleLead)><code>multiple</code> when several may stay open together.</p>
        <code-block code={accExMultiple()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
