<!--
  <doc-table> — /components/table
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<controller>
import type { Scope } from './doc-table.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const dense = signal(false)
  const striped = signal(false)
  const hoverable = signal(false)
  const bordered = signal(false)
  const people = () => [
    { id: '1', name: 'Ada Lovelace', team: 'Math', status: 'Active' },
    { id: '2', name: 'Grace Hopper', team: 'Navy', status: 'Active' },
    { id: '3', name: 'Katherine Johnson', team: 'NASA', status: 'Active' },
  ]

  const entry = DETAILS.table
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = []
    if (dense()) a.push('dense')
    if (striped()) a.push('striped')
    if (hoverable()) a.push('hoverable')
    if (bordered()) a.push('bordered')
    const open = a.length ? `<table ${a.join(' ')}>` : `<table>`
    return `${open}
  <thead>
    <tr><th>Name</th><th>Team</th><th>Status</th></tr>
  </thead>
  <tbody>
    <tr><td>Ada Lovelace</td><td>Math</td><td>Active</td></tr>
    <tr><td>Grace Hopper</td><td>Navy</td><td>Active</td></tr>
  </tbody>
</table>`
  })

  const tbExDefault = () =>
    `<table>
  <thead><tr><th>Name</th><th>Team</th><th>Status</th></tr></thead>
  <tbody>
    <tr><td>Ada</td><td>Math</td><td>Active</td></tr>
  </tbody>
</table>`
  const tbExStriped = () =>
    `<table striped hoverable>
  <thead>…</thead>
  <tbody>…</tbody>
</table>`
  const tbExDense = () =>
    `<table dense bordered>
  <thead>…</thead>
  <tbody>…</tbody>
</table>`

  return {
    dense,
    striped,
    hoverable,
    bordered,
    people,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    tbExDefault,
    tbExStriped,
    tbExDense,
  }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style specimen = { box: w(full).maxW(panel-max-w-md).pad(2).overflow(visible) }
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
  style tableW = { box: w(full).maxW(panel-max-w-md) }
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
      <table
        dense={dense() ? 'true' : null}
        striped={striped() ? 'true' : null}
        hoverable={hoverable() ? 'true' : null}
        bordered={bordered() ? 'true' : null}
        style(tableW)
      >
        <thead>
          <tr><th>Name</th><th>Team</th><th>Status</th></tr>
        </thead>
        <tbody>
          <for {people() as p} key={p.id}>
            <tr>
              <td>{p.name}</td>
              <td>{p.team}</td>
              <td>{p.status}</td>
            </tr>
          </for>
        </tbody>
      </table>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Dense</label>
        <row gap="2" align="center">
          <switch bind:checked={dense} size="sm" label="Dense"></switch>
          <span style(sizeValue)>{dense() ? 'tight pad' : 'default pad'}</span>
        </row>
      </column>
      <column gap="2">
        <label style(knobLabel)>Striped</label>
        <row gap="2" align="center">
          <switch bind:checked={striped} size="sm" label="Striped"></switch>
          <span style(sizeValue)>{striped() ? 'even rows stripe' : 'flat rows'}</span>
        </row>
      </column>
      <column gap="2">
        <label style(knobLabel)>Hoverable</label>
        <row gap="2" align="center">
          <switch bind:checked={hoverable} size="sm" label="Hoverable"></switch>
          <span style(sizeValue)>{hoverable() ? 'row hover' : 'static'}</span>
        </row>
      </column>
      <column gap="2">
        <label style(knobLabel)>Bordered</label>
        <row gap="2" align="center">
          <switch bind:checked={bordered} size="sm" label="Bordered"></switch>
          <span style(sizeValue)>{bordered() ? 'cell box' : 'row lines'}</span>
        </row>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="stretch" style(specimen)>
      <table style(tableW)>
        <thead>
          <tr><th>Name</th><th>Team</th><th>Status</th></tr>
        </thead>
        <tbody>
          <tr><td>Ada Lovelace</td><td>Math</td><td>Active</td></tr>
          <tr><td>Grace Hopper</td><td>Navy</td><td>Active</td></tr>
        </tbody>
      </table>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Default table"
        lead="Plain thead / tbody / tr — header divided, rows separated, no host flags."
        code={tbExDefault()}
      >
        <table style(tableW)>
          <thead>
            <tr><th>Name</th><th>Team</th><th>Status</th></tr>
          </thead>
          <tbody>
            <tr><td>Ada Lovelace</td><td>Math</td><td>Active</td></tr>
            <tr><td>Grace Hopper</td><td>Navy</td><td>Active</td></tr>
          </tbody>
        </table>
      </doc-example>
      <doc-example
        title="Striped + hoverable"
        lead="Even rows get a light stripe; hover reuses the same fill."
        code={tbExStriped()}
      >
        <table striped hoverable style(tableW)>
          <thead>
            <tr><th>Name</th><th>Team</th><th>Status</th></tr>
          </thead>
          <tbody>
            <tr><td>Ada Lovelace</td><td>Math</td><td>Active</td></tr>
            <tr><td>Grace Hopper</td><td>Navy</td><td>Active</td></tr>
            <tr><td>Katherine Johnson</td><td>NASA</td><td>Active</td></tr>
          </tbody>
        </table>
      </doc-example>
      <doc-example
        title="Dense + bordered"
        lead="Tighter cell padding and a full cell box — structural chrome only, not a data grid."
        code={tbExDense()}
      >
        <table dense bordered style(tableW)>
          <thead>
            <tr><th>Name</th><th>Email</th><th numeric>Amount</th></tr>
          </thead>
          <tbody>
            <tr><td>Ada</td><td>ada@ship.dev</td><td numeric>1250</td></tr>
            <tr><td>Grace</td><td>grace@ship.dev</td><td numeric>2500</td></tr>
          </tbody>
        </table>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
