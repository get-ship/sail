<!--
  <doc-table> — /components/table
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { size({ w: 100%, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style specimen = { size({ w: 100%, max-w: overlay-max-w-md }).pad(2).overflow({ all: visible }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style sizeValue = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style exampleBlock = { size({ w: 100% }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
  style tableW = { size({ w: 100%, max-w: overlay-max-w-md }) }
</scope>

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
        lead="Plain thead / tbody / tr — outer hairline and row lines on by default."
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
