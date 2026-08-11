<!--
  <doc-empty-state> — first-class docs host for /components/empty-state.
  Specimen left, knobs right — variant × size × optional tone.
-->

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
  const esMeta = derived(() => {
    const t = esTone()
    const live = variant() === 'connecting' ? 'status/polite' : 'note'
    return `${variant()} · ${esSize()} · ${t || 'rest icon'} · ${live} · component.empty-state.*`
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
  const esExDefault = () =>
    `<empty-state>\n  <icon slot="icon" name="inbox"></icon>\n  <strong slot="title">No results</strong>\n  Try a different search.\n  <button slot="action" variant="outline" size="sm">Clear filters</button>\n</empty-state>`
  const esExConnecting = () =>
    `<empty-state variant="connecting">\n  <icon slot="icon" name="refresh-cw" spin></icon>\n  <strong slot="title">Connecting…</strong>\n  Establishing a secure session.\n</empty-state>`
  const esExRest = () =>
    `<empty-state>\n  <strong slot="title">Empty</strong>\n  Omit tone → component.empty-state.* icon rest.\n</empty-state>`
  return {
    sizeIdx,
    variant,
    esTone,
    pickEsTone,
    clearEsTone,
    esSize,
    esMeta,
    esTitle,
    esBody,
    esIcon,
    heading,
    blurb,
    props,
    usageCode,
    esExDefault,
    esExConnecting,
    esExRest,
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
  style tickRow = { box: w(full).padT(1) }
  style tick = {
    box: grow(1).basis(0).minW(0),
    paint: fg(muted-foreground),
    text: fontSize(xs).fontFamily(mono).textAlign(center),
  }
  style swatchRow = { box: w(full) }
  style swatch = {
    box: w(7).h(7).shrink(0),
    paint: radius(full).cursor(pointer).borderWidth(2).border(transparent).shadow(xs),
    when: {
      selected: { paint: border(foreground).shadow(sm) },
      hover: { paint: opacity(90) },
    },
  }
  style swatchPrimary = { paint: bg(primary) }
  style swatchSuccess = { paint: bg(success) }
  style swatchWarning = { paint: bg(warning) }
  style swatchDestructive = { paint: bg(destructive) }
  style swatchAccent = { paint: bg(accent) }
  style swatchMuted = { paint: bg(muted).border(border) }
  style swatchInfo = { paint: bg(info) }
  style swatchRest = { paint: bg(background).border(border) }
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
            <if {esTone()}>
              <empty-state variant={variant()} size={esSize()} tone={esTone()} style(fullW)>
                <icon slot="icon" name={esIcon()} spin={variant() === 'connecting' ? 'true' : null}></icon>
                <strong slot="title">{esTitle()}</strong>
                {esBody()}
                <if {variant() !== 'connecting'}>
                  <button slot="action" variant="outline" size="sm">Clear filters</button>
                </if>
              </empty-state>
            {else =>
              <empty-state variant={variant()} size={esSize()} style(fullW)>
                <icon slot="icon" name={esIcon()} spin={variant() === 'connecting' ? 'true' : null}></icon>
                <strong slot="title">{esTitle()}</strong>
                {esBody()}
                <if {variant() !== 'connecting'}>
                  <button slot="action" variant="outline" size="sm">Clear filters</button>
                </if>
              </empty-state>}</if>
            <p style(monoXs)>{esMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Variant</label>
            <select size="sm" bind:value={variant} label="Variant">
              <option value="default">default</option>
              <option value="inline">inline</option>
              <option value="placeholder">placeholder</option>
              <option value="connecting">connecting</option>
            </select>
            <p style(mutedXs)>
              default bordered · inline dashed · placeholder/connecting bare.
              <code>connecting</code> → role=status / aria-live=polite.
            </p>
          </column>
          <column gap="2">
            <row gap="2" align="center" justify="between">
              <label style(knobLabel)>Size</label>
              <span style(sizeValue)>{esSize()}</span>
            </row>
            <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size"></slider>
            <row gap="0" align="center" style(tickRow)>
              <span style(tick)>sm</span>
              <span style(tick)>md</span>
              <span style(tick)>lg</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Tone</label>
            <row gap="2" align="center" style(swatchRow)>
              <tooltip content="rest — component.empty-state icon" placement="top">
                <div label="rest" on:click={clearEsTone} selected={!esTone() ? 'true' : null} style(swatch, swatchRest)></div>
              </tooltip>
              <tooltip content="primary" placement="top">
                <div label="primary" on:click={() => pickEsTone('primary')} selected={esTone() === 'primary' ? 'true' : null} style(swatch, swatchPrimary)></div>
              </tooltip>
              <tooltip content="success" placement="top">
                <div label="success" on:click={() => pickEsTone('success')} selected={esTone() === 'success' ? 'true' : null} style(swatch, swatchSuccess)></div>
              </tooltip>
              <tooltip content="warning" placement="top">
                <div label="warning" on:click={() => pickEsTone('warning')} selected={esTone() === 'warning' ? 'true' : null} style(swatch, swatchWarning)></div>
              </tooltip>
              <tooltip content="destructive" placement="top">
                <div label="destructive" on:click={() => pickEsTone('destructive')} selected={esTone() === 'destructive' ? 'true' : null} style(swatch, swatchDestructive)></div>
              </tooltip>
              <tooltip content="accent" placement="top">
                <div label="accent" on:click={() => pickEsTone('accent')} selected={esTone() === 'accent' ? 'true' : null} style(swatch, swatchAccent)></div>
              </tooltip>
              <tooltip content="muted" placement="top">
                <div label="muted" on:click={() => pickEsTone('muted')} selected={esTone() === 'muted' ? 'true' : null} style(swatch, swatchMuted)></div>
              </tooltip>
              <tooltip content="info" placement="top">
                <div label="info" on:click={() => pickEsTone('info')} selected={esTone() === 'info' ? 'true' : null} style(swatch, swatchInfo)></div>
              </tooltip>
            </row>
            <p style(mutedXs)>Tone brands the icon only. Omit for themeable rest chrome.</p>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <column gap="2">
        <h3 style(exampleH)>Default card</h3>
        <p style(exampleLead)>
          Icon + title + body + action slots. Empty slots collapse.
        </p>
        <empty-state style(stageCol)>
          <icon slot="icon" name="inbox"></icon>
          <strong slot="title">No results</strong>
          Try a different search, or clear a filter.
          <button slot="action" variant="outline" size="sm">Clear filters</button>
        </empty-state>
        <code-block code={esExDefault()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Connecting (live status)</h3>
        <p style(exampleLead)>
          <code>variant="connecting"</code> upgrades to role=status / aria-live=polite
          via <code>use:variantLive</code>.
        </p>
        <empty-state variant="connecting" style(stageCol)>
          <icon slot="icon" name="refresh-cw" spin></icon>
          <strong slot="title">Connecting…</strong>
          Establishing a secure session.
        </empty-state>
        <code-block code={esExConnecting()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Rest tokens (themeable)</h3>
        <p style(exampleLead)>
          Omit tone → <code>component.empty-state.bg</code> · <code>border</code> ·
          <code>fg</code> · <code>title</code>.
        </p>
        <empty-state style(stageCol)>
          <strong slot="title">Empty</strong>
          Theme the leaves for every rest empty-state.
        </empty-state>
        <code-block code={esExRest()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
