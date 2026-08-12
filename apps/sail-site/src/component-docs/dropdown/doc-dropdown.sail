<!--
  <doc-dropdown> — /components/dropdown
  Shared chrome: split + snippet copy + usage Code|Live|Split.
  Specimen: invoker + placement · onSelect live.
-->

<controller>
import type { Scope } from './doc-dropdown.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const placement = signal('bottom-start')
  const last = signal('')
  const onSelect = (detail: { value?: string }): void => {
    last.set(detail?.value ?? '')
  }

  const entry = DETAILS.dropdown
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = ['id="menu"']
    if (placement() !== 'bottom') a.push(`placement="${placement()}"`)
    a.push('on-select={onSelect}')
    return `<button popovertarget="menu">Actions</button>\n<dropdown ${a.join(' ')}>\n  <menu-item value="edit">Edit</menu-item>\n  <menu-item value="dup">Duplicate</menu-item>\n</dropdown>`
  })

  const ddExBasic = () =>
    `<button popovertarget="menu">Actions</button>\n<dropdown id="menu" placement="bottom-start" on-select={onSelect}>\n  <menu-item value="edit">Edit</menu-item>\n  <menu-item value="dup">Duplicate</menu-item>\n</dropdown>`
  const ddExDanger = () =>
    `<dropdown id="m" placement="bottom">\n  <menu-item value="del" tone="destructive">Delete</menu-item>\n</dropdown>`
  const ddExPlace = () =>
    `<dropdown id="m" placement="top-end">…</dropdown>`

  return {
    placement,
    last,
    onSelect,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    ddExBasic,
    ddExDanger,
    ddExPlace,
  }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style specimen = { box: pad(2).overflow(visible) }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
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
    <column slot="specimen" gap="3" align="center" style(specimen)>
      <button size="sm" variant="outline" popovertarget="doc-dd-live">Row actions ▾</button>
      <dropdown id="doc-dd-live" placement={placement()} on-select={onSelect}>
        <menu-item value="edit">Edit</menu-item>
        <menu-item value="duplicate">Duplicate</menu-item>
        <menu-item value="archive">Archive</menu-item>
      </dropdown>
      <if {last()}>
        <p style(monoXs)>selected {last()}</p>
      </if>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Placement</label>
        <select size="sm" bind:value={placement} label="Placement">
          <option value="bottom">bottom</option>
          <option value="bottom-start">bottom-start</option>
          <option value="top">top</option>
          <option value="top-end">top-end</option>
        </select>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="center" style(specimen)>
      <button size="sm" variant="outline" popovertarget="doc-dd-usage">Actions ▾</button>
      <dropdown id="doc-dd-usage" placement="bottom-start" on-select={onSelect}>
        <menu-item value="edit">Edit</menu-item>
        <menu-item value="dup">Duplicate</menu-item>
      </dropdown>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Actions menu"
        lead="Roving arrows, type-ahead, select closes + emits."
        code={ddExBasic()}
      >
        <button size="sm" variant="outline" popovertarget="doc-dd-ex1">Actions ▾</button>
        <dropdown id="doc-dd-ex1" placement="bottom-start" on-select={onSelect}>
          <menu-item value="edit">Edit</menu-item>
          <menu-item value="dup">Duplicate</menu-item>
        </dropdown>
      </doc-example>
      <doc-example
        title="Destructive item"
        lead="tone on menu-item for danger rows."
        code={ddExDanger()}
      >
        <button size="sm" variant="outline" popovertarget="doc-dd-ex2">Danger ▾</button>
        <dropdown id="doc-dd-ex2" placement="bottom">
          <menu-item value="del" tone="destructive">Delete</menu-item>
        </dropdown>
      </doc-example>
      <doc-example
        title="Placement string"
        lead="Free host string — common anchors in knobs."
        code={ddExPlace()}
      >
        <button size="sm" variant="outline" popovertarget="doc-dd-ex3">top-end ▾</button>
        <dropdown id="doc-dd-ex3" placement="top-end">
          <menu-item value="a">Option A</menu-item>
          <menu-item value="b">Option B</menu-item>
        </dropdown>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
