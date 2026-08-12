<!--
  <doc-table> — first-class docs host for /components/table.
  Specimen left, knobs right — dense · striped · hoverable · bordered.
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
  const tbMeta = derived(() => {
    const flags = [
      dense() ? 'dense' : null,
      striped() ? 'striped' : null,
      hoverable() ? 'hoverable' : null,
      bordered() ? 'bordered' : null,
    ].filter(Boolean)
    const f = flags.length ? flags.join(' · ') : 'rest'
    return `${f} · component.table.fg / muted-fg / stripe / sticky-bg`
  })
  const entry = DETAILS.table
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const tbExDefault = () =>
    `<table>\n  <thead><tr><th>Name</th><th>Team</th><th>Status</th></tr></thead>\n  <tbody>\n    <tr><td>Ada</td><td>Math</td><td>Active</td></tr>\n  </tbody>\n</table>`
  const tbExStriped = () =>
    `<table striped hoverable>\n  <thead>…</thead>\n  <tbody>…</tbody>\n</table>`
  const tbExDense = () =>
    `<table dense bordered>\n  <thead>…</thead>\n  <tbody>…</tbody>\n</table>`
  return {
    dense,
    striped,
    hoverable,
    bordered,
    people,
    tbMeta,
    heading,
    blurb,
    props,
    usageCode,
    tbExDefault,
    tbExStriped,
    tbExDense,
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
            <table
              dense={dense() ? 'true' : null}
              striped={striped() ? 'true' : null}
              hoverable={hoverable() ? 'true' : null}
              bordered={bordered() ? 'true' : null}
              style(stageCol)
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
            <p style(monoXs)>{tbMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Dense</label>
            <row gap="2" align="center">
              <switch bind:checked={dense} size="sm" tone="primary" label="Dense"></switch>
              <span style(sizeValue)>{dense() ? 'tight pad' : 'default pad'}</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Striped</label>
            <row gap="2" align="center">
              <switch bind:checked={striped} size="sm" tone="primary" label="Striped"></switch>
              <span style(sizeValue)>{striped() ? 'even rows stripe' : 'flat rows'}</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Hoverable</label>
            <row gap="2" align="center">
              <switch bind:checked={hoverable} size="sm" tone="primary" label="Hoverable"></switch>
              <span style(sizeValue)>{hoverable() ? 'row hover' : 'static'}</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Bordered</label>
            <row gap="2" align="center">
              <switch bind:checked={bordered} size="sm" tone="primary" label="Bordered"></switch>
              <span style(sizeValue)>{bordered() ? 'cell box' : 'row lines'}</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <column gap="1" style(faceRow)>
              <p style(mutedXs)><code>component.table.fg</code> — body ink</p>
              <p style(mutedXs)><code>component.table.muted-fg</code> — header ink</p>
              <p style(mutedXs)><code>component.table.stripe</code> — even / hover fill</p>
              <p style(mutedXs)><code>component.table.sticky-bg</code> — sticky thead</p>
            </column>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <column gap="2">
        <h3 style(exampleH)>Default table</h3>
        <p style(exampleLead)>
          Platform thead / tbody / tr — header divided, rows separated, no host flags.
        </p>
        <table style(stageCol)>
          <thead>
            <tr><th>Name</th><th>Team</th><th>Status</th></tr>
          </thead>
          <tbody>
            <tr><td>Ada Lovelace</td><td>Math</td><td>Active</td></tr>
            <tr><td>Grace Hopper</td><td>Navy</td><td>Active</td></tr>
          </tbody>
        </table>
        <code-block code={tbExDefault()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Striped + hoverable</h3>
        <p style(exampleLead)>
          Even rows sample <code>component.table.stripe</code>; hover reuses the same leaf.
        </p>
        <table striped hoverable style(stageCol)>
          <thead>
            <tr><th>Name</th><th>Team</th><th>Status</th></tr>
          </thead>
          <tbody>
            <tr><td>Ada Lovelace</td><td>Math</td><td>Active</td></tr>
            <tr><td>Grace Hopper</td><td>Navy</td><td>Active</td></tr>
            <tr><td>Katherine Johnson</td><td>NASA</td><td>Active</td></tr>
          </tbody>
        </table>
        <code-block code={tbExStriped()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Dense + bordered</h3>
        <p style(exampleLead)>
          Tighter cell pad and a full cell box — structural host Style only, not a data grid.
        </p>
        <table dense bordered style(stageCol)>
          <thead>
            <tr><th>Name</th><th>Email</th><th numeric>Amount</th></tr>
          </thead>
          <tbody>
            <tr><td>Ada</td><td>ada@ship.dev</td><td numeric>1250</td></tr>
            <tr><td>Grace</td><td>grace@ship.dev</td><td numeric>2500</td></tr>
          </tbody>
        </table>
        <code-block code={tbExDense()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
