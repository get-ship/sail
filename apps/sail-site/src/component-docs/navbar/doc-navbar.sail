<!--
  <doc-navbar> — /components/navbar
  Shared chrome: split + snippet copy + usage Code|Live|Split.
  Specimen: placement · sticky · bordered · autohide · offset knobs.
-->

<controller>
import type { Scope } from './doc-navbar.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const placement = signal('top')
  const sticky = signal(false)
  const bordered = signal(true)
  const autohide = signal(false)
  const offset = signal('0')

  const entry = DETAILS.navbar
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = [`placement="${placement()}"`]
    if (sticky()) a.push('sticky')
    if (bordered()) a.push('bordered')
    if (autohide()) a.push('autohide')
    if (offset() !== '0') a.push(`offset="${offset()}"`)
    return `<navbar ${a.join(' ')}>\n  <strong slot="brand">Ship</strong>\n  <a href="/docs">Docs</a>\n  <button slot="actions" size="sm" variant="outline">Sign in</button>\n</navbar>`
  })

  const navExTop = () =>
    `<navbar placement="top" bordered>\n  <strong slot="brand">Ship</strong>\n  <a href="/docs">Docs</a>\n  <button slot="actions" size="sm" variant="outline">Sign in</button>\n</navbar>`
  const navExRail = () =>
    `<navbar placement="rail" bordered sticky>\n  <strong slot="brand">Ship</strong>\n  <a href="/docs">Docs</a>\n</navbar>`
  const navExBottom = () =>
    `<navbar placement="bottom" sticky bordered>\n  <strong slot="brand">Ship</strong>\n  <button slot="actions" size="sm">Account</button>\n</navbar>`

  return {
    placement,
    sticky,
    bordered,
    autohide,
    offset,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    navExTop,
    navExRail,
    navExBottom,
  }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style specimen = { box: w(full).pad(2).overflow(visible) }
  style fullW = { box: w(full) }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style exampleBlock = { box: w(full).padT(2).padB(4).overflow(visible) }
  style sectionLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).textCase(uppercase).tracking(wide),
  }
</scope>

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
      <navbar
        placement={placement()}
        sticky={sticky() ? 'true' : null}
        bordered={bordered() ? 'true' : null}
        autohide={autohide() ? 'true' : null}
        offset={offset()}
        style(fullW)
      >
        <strong slot="brand">Ship</strong>
        <a href="/components">Components</a>
        <a href="/docs">Docs</a>
        <button slot="actions" size="sm" variant="outline">Sign in</button>
      </navbar>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Placement</label>
        <select size="sm" bind:value={placement} label="Placement">
          <option value="top">top</option>
          <option value="bottom">bottom</option>
          <option value="rail">rail</option>
        </select>
      </column>
      <column gap="2">
        <label style(knobLabel)>Sticky</label>
        <row gap="2" align="center">
          <switch bind:checked={sticky} size="sm" tone="primary" label="Sticky"></switch>
          <span style(sizeValue)>{sticky() ? 'pinned' : 'flow'}</span>
        </row>
      </column>
      <column gap="2">
        <label style(knobLabel)>Bordered</label>
        <row gap="2" align="center">
          <switch bind:checked={bordered} size="sm" tone="primary" label="Bordered"></switch>
          <span style(sizeValue)>{bordered() ? 'edge hairline' : 'flat'}</span>
        </row>
      </column>
      <column gap="2">
        <label style(knobLabel)>Autohide</label>
        <row gap="2" align="center">
          <switch bind:checked={autohide} size="sm" tone="primary" label="Autohide"></switch>
          <span style(sizeValue)>{autohide() ? 'tuck on scroll' : 'always'}</span>
        </row>
        <p style(mutedXs)>With sticky + chromeOffset — operator behavior depth later.</p>
      </column>
      <column gap="2">
        <label style(knobLabel)>Offset</label>
        <select size="sm" bind:value={offset} label="Offset">
          <option value="0">0</option>
          <option value="8">8</option>
          <option value="16">16</option>
          <option value="48">48</option>
        </select>
        <p style(mutedXs)>Sticky top/left inset (number).</p>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="stretch" style(specimen)>
      <navbar placement="top" bordered style(fullW)>
        <strong slot="brand">Ship</strong>
        <a href="/docs">Docs</a>
        <button slot="actions" size="sm" variant="outline">Sign in</button>
      </navbar>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Top bar"
        lead="Default placement. Brand · links · actions regions."
        code={navExTop()}
      >
        <navbar placement="top" bordered style(fullW)>
          <strong slot="brand">Ship</strong>
          <a href="/docs">Docs</a>
          <button slot="actions" size="sm" variant="outline">Sign in</button>
        </navbar>
      </doc-example>
      <doc-example
        title="Rail"
        lead={'placement="rail" — side nav geometry.'}
        code={navExRail()}
      >
        <navbar placement="rail" bordered style({ box: minH(40) })>
          <strong slot="brand">Ship</strong>
          <a href="/docs">Docs</a>
          <a href="/components">Components</a>
        </navbar>
      </doc-example>
      <doc-example
        title="Bottom sticky"
        lead="Bottom placement pins with sticky."
        code={navExBottom()}
      >
        <navbar placement="bottom" sticky bordered style(fullW)>
          <strong slot="brand">Ship</strong>
          <button slot="actions" size="sm">Account</button>
        </navbar>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
