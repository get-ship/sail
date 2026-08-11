<!--
  <doc-card> — first-class docs host for /components/card.
  Specimen left, knobs right — variant · padding · tone · interactive · accent-bar · status-rail · fill.
  NO invent soft/elevated/ghost.
-->

<controller>
import type { Scope } from './doc-card.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const variant = signal('default')
  const padding = signal('md')
  const cardTone = signal('')
  const interactive = signal(false)
  const accentBar = signal('off')
  const statusRail = signal(false)
  const fill = signal(false)
  const pickTone = (t: string): void => cardTone.set(t)
  const clearTone = (): void => cardTone.set('')
  const cardMeta = derived(() => {
    const t = cardTone() || 'rest'
    const flags = [
      interactive() ? 'interactive' : null,
      statusRail() ? 'status-rail' : null,
      fill() ? 'fill' : null,
    ].filter(Boolean)
    const f = flags.length ? ` · ${flags.join(' · ')}` : ''
    return `${variant()} · pad ${padding()} · tone ${t} · accent-bar ${accentBar()}${f}`
  })
  const entry = DETAILS.card
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const cardExDefault = () =>
    `<card>\n  <strong>Harbor Signals</strong>\n  <p>4 tracks · last played today</p>\n</card>`
  const cardExInteractive = () =>
    `<card interactive accent-bar="static" tone="primary">\n  <strong>Open project</strong>\n</card>`
  const cardExOutline = () =>
    `<card variant="outline" padding="lg" status-rail>\n  <strong>Outline shell</strong>\n</card>`
  return {
    variant, padding, cardTone, interactive, accentBar, statusRail, fill,
    pickTone, clearTone, cardMeta,
    heading, blurb, props, usageCode,
    cardExDefault, cardExInteractive, cardExOutline,
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
  style swatchRow = { box: w(full) }
  style swatch = {
    box: size(6).shrink(0),
    paint: radius(full).borderWidth(1).border(border).cursor(pointer),
  }
  style swatchRest = { paint: bg(surface-1) }
  style swatchPrimary = { paint: bg(primary) }
  style swatchInfo = { paint: bg(info) }
  style swatchSuccess = { paint: bg(success) }
  style swatchWarning = { paint: bg(warning) }
  style swatchDestructive = { paint: bg(destructive) }
  style swatchAccent = { paint: bg(accent) }
  style swatchMuted = { paint: bg(muted) }
  style centerBox = { box: pad(4), paint: borderWidth(1).border(border-subtle).radius(md) }

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
            <if {cardTone()}>
              <card
                variant={variant()}
                padding={padding()}
                tone={cardTone()}
                interactive={interactive() ? 'true' : null}
                accent-bar={accentBar()}
                status-rail={statusRail() ? 'true' : null}
                fill={fill() ? 'true' : null}
                style(stageCol)
              >
                <strong>Harbor Signals</strong>
                <p style(mutedXs)>4 tracks · last played today</p>
              </card>
            {else =>
              <card
                variant={variant()}
                padding={padding()}
                interactive={interactive() ? 'true' : null}
                accent-bar={accentBar()}
                status-rail={statusRail() ? 'true' : null}
                fill={fill() ? 'true' : null}
                style(stageCol)
              >
                <strong>Harbor Signals</strong>
                <p style(mutedXs)>4 tracks · last played today</p>
              </card>}</if>
            <p style(monoXs)>{cardMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Variant</label>
            <select size="sm" bind:value={variant} label="Variant">
              <option value="default">default</option>
              <option value="outline">outline</option>
            </select>
            <p style(mutedXs)>default | outline only — no soft/elevated/ghost invent.</p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Padding</label>
            <select size="sm" bind:value={padding} label="Padding">
              <option value="none">none</option>
              <option value="sm">sm</option>
              <option value="md">md</option>
              <option value="lg">lg</option>
            </select>
          </column>
          <column gap="2">
            <label style(knobLabel)>Tone</label>
            <row gap="2" align="center" style(swatchRow)>
              <tooltip content="rest — component.card tokens" placement="top">
                <div label="rest" on:click={clearTone} selected={!cardTone() ? 'true' : null} style(swatch, swatchRest)></div>
              </tooltip>
              <tooltip content="primary" placement="top">
                <div label="primary" on:click={() => pickTone('primary')} selected={cardTone() === 'primary' ? 'true' : null} style(swatch, swatchPrimary)></div>
              </tooltip>
              <tooltip content="info" placement="top">
                <div label="info" on:click={() => pickTone('info')} selected={cardTone() === 'info' ? 'true' : null} style(swatch, swatchInfo)></div>
              </tooltip>
              <tooltip content="success" placement="top">
                <div label="success" on:click={() => pickTone('success')} selected={cardTone() === 'success' ? 'true' : null} style(swatch, swatchSuccess)></div>
              </tooltip>
              <tooltip content="warning" placement="top">
                <div label="warning" on:click={() => pickTone('warning')} selected={cardTone() === 'warning' ? 'true' : null} style(swatch, swatchWarning)></div>
              </tooltip>
              <tooltip content="destructive" placement="top">
                <div label="destructive" on:click={() => pickTone('destructive')} selected={cardTone() === 'destructive' ? 'true' : null} style(swatch, swatchDestructive)></div>
              </tooltip>
              <tooltip content="accent" placement="top">
                <div label="accent" on:click={() => pickTone('accent')} selected={cardTone() === 'accent' ? 'true' : null} style(swatch, swatchAccent)></div>
              </tooltip>
              <tooltip content="muted" placement="top">
                <div label="muted" on:click={() => pickTone('muted')} selected={cardTone() === 'muted' ? 'true' : null} style(swatch, swatchMuted)></div>
              </tooltip>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Interactive</label>
            <row gap="2" align="center">
              <switch bind:checked={interactive} size="sm" tone="primary" label="Interactive"></switch>
              <span style(sizeValue)>{interactive() ? 'hover lift' : 'static'}</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Accent bar</label>
            <select size="sm" bind:value={accentBar} label="Accent bar">
              <option value="off">off</option>
              <option value="hover">hover</option>
              <option value="static">static</option>
            </select>
          </column>
          <column gap="2">
            <label style(knobLabel)>Status rail</label>
            <row gap="2" align="center">
              <switch bind:checked={statusRail} size="sm" tone="primary" label="Status rail"></switch>
              <span style(sizeValue)>{statusRail() ? 'leading rail' : 'off'}</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Fill</label>
            <row gap="2" align="center">
              <switch bind:checked={fill} size="sm" tone="primary" label="Fill"></switch>
              <span style(sizeValue)>{fill() ? 'muted fill' : 'token fill'}</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <column gap="1" style(faceRow)>
              <p style(mutedXs)><code>component.card.bg</code> · <code>fg</code></p>
              <p style(mutedXs)><code>texture*</code> · <code>backdrop-filter</code></p>
            </column>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Default surface</h3>
        <p style(exampleLead)>Rest fill from <code>component.card.bg</code>.</p>
        <card style(stageCol)>
          <strong>Harbor Signals</strong>
          <p style(mutedXs)>4 tracks · last played today</p>
        </card>
        <code-block code={cardExDefault()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Interactive + accent</h3>
        <p style(exampleLead)><code>interactive</code> hover lift; <code>accent-bar="static"</code> with tone.</p>
        <card interactive accent-bar="static" tone="primary" style(stageCol)>
          <strong>Open project</strong>
          <p style(mutedXs)>Pointer + lift, one attribute.</p>
        </card>
        <code-block code={cardExInteractive()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Outline + status rail</h3>
        <p style(exampleLead)><code>variant="outline"</code> and <code>status-rail</code> — no soft/elevated invent.</p>
        <card variant="outline" padding="lg" status-rail style(stageCol)>
          <strong>Outline shell</strong>
          <p style(mutedXs)>Transparent body, leading rail.</p>
        </card>
        <code-block code={cardExOutline()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
