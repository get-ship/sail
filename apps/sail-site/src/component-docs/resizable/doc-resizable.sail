<!--
  <doc-resizable> — /components/resizable
  Shared chrome: split + snippet copy + usage Code|Live|Split.
  Specimen: two panes (start / end slots). Orientation knob only.
-->

<scope>
  style page = { size({ w: 100%, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style mutedSm = { fg(muted-foreground).type({ size: sm }) }
  style mutedXs = { fg(muted-foreground).type({ size: xs }) }
  style specimen = { size({ w: 100%, max-w: overlay-max-w-md }).pad(2).overflow({ all: visible }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style exampleBlock = { size({ w: 100% }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
  style splitFrame = { size({ w: 100%, h: 48 }).overflow({ all: hidden }).border({ color: border-subtle, width: 1 }).radius(md) }
  style paneStart = { size({ w: 100%, h: 100% }).pad(3).bg(surface-1) }
  style paneEnd = { size({ w: 100%, h: 100% }).pad(3).bg(surface-2) }
</scope>

<controller>
import type { Scope } from './doc-resizable.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const orientation = signal('horizontal')

  const entry = DETAILS.resizable
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const o = orientation() !== 'horizontal' ? ` orientation="${orientation()}"` : ''
    return `<resizable${o}>\n  <div slot="start">Start pane</div>\n  <div slot="end">End pane</div>\n</resizable>`
  })

  const rzExHoriz = () =>
    `<resizable>\n  <div slot="start">Start pane</div>\n  <div slot="end">End pane</div>\n</resizable>`
  const rzExVert = () =>
    `<resizable orientation="vertical">\n  <div slot="start">Top pane</div>\n  <div slot="end">Bottom pane</div>\n</resizable>`

  return {
    orientation,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    rzExHoriz,
    rzExVert,
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
      <div style(splitFrame)>
        <resizable orientation={orientation()}>
          <div slot="start" style(paneStart)>
            <p style(mutedSm)>Start pane</p>
          </div>
          <div slot="end" style(paneEnd)>
            <p style(mutedSm)>End pane</p>
          </div>
        </resizable>
      </div>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Orientation</label>
        <select size="sm" bind:value={orientation} label="Orientation">
          <option value="horizontal">horizontal</option>
          <option value="vertical">vertical</option>
        </select>
        <p style(mutedXs)>
          horizontal splits start|end on the inline axis. vertical stacks them.
          Drag the separator — or focus it and use the arrow keys.
        </p>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="stretch" style(specimen)>
      <div style(splitFrame)>
        <resizable orientation="horizontal">
          <div slot="start" style(paneStart)>
            <p style(mutedSm)>Start pane</p>
          </div>
          <div slot="end" style(paneEnd)>
            <p style(mutedSm)>End pane</p>
          </div>
        </resizable>
      </div>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Horizontal split"
        lead="Default orientation. Drag the handle to size the start pane."
        code={rzExHoriz()}
      >
        <div style(splitFrame)>
          <resizable orientation="horizontal">
            <div slot="start" style(paneStart)>
              <p style(mutedSm)>Start pane</p>
            </div>
            <div slot="end" style(paneEnd)>
              <p style(mutedSm)>End pane</p>
            </div>
          </resizable>
        </div>
      </doc-example>
      <doc-example
        title="Vertical split"
        lead={'orientation="vertical" — top / bottom panes.'}
        code={rzExVert()}
      >
        <div style(splitFrame)>
          <resizable orientation="vertical">
            <div slot="start" style(paneStart)>
              <p style(mutedSm)>Top pane</p>
            </div>
            <div slot="end" style(paneEnd)>
              <p style(mutedSm)>Bottom pane</p>
            </div>
          </resizable>
        </div>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components/scroll-area">Scroll area</a>
  </row>
</column>
