<!--
  <doc-tooltip> — /components/tooltip
  Shared chrome: split + snippet copy + usage Code|Live|Split.
  Specimen: hover/focus trigger with placement · content knobs.
-->

<scope>
  style page = { box: size({ w: full, max-w: breakpoint-desktop }).space({ mx: auto }).overflow({ all: visible }) }
  style mutedSm = { paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
  style specimen = { box: space({ pad: 2 }).overflow({ all: visible }) }
  style knobLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, weight: medium, tracking: wide, case: uppercase }),
}
  style exampleBlock = { box: size({ w: full }).space({ pad-t: 2, pad-b: 4 }).overflow({ all: visible }) }
  style sectionLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, tracking: wide, case: uppercase }),
}
  style demoRow = { box: size({ w: full }).space({ pad-y: 2 }) }
</scope>

<controller>
import type { Scope } from './doc-tooltip.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const placement = signal('top')
  const content = signal('Copy URL')

  const entry = DETAILS.tooltip
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = [`content="${content()}"`]
    if (placement() !== 'top') a.push(`placement="${placement()}"`)
    return `<tooltip ${a.join(' ')}>\n  <span>ship.dev/d/ab12</span>\n</tooltip>`
  })

  const tipExTop = () =>
    `<tooltip content="Copy URL" placement="top">\n  <span>ship.dev/d/ab12</span>\n</tooltip>`
  const tipExSides = () =>
    `<tooltip content="Top" placement="top">…</tooltip>\n<tooltip content="Bottom" placement="bottom">…</tooltip>`
  const tipExDanger = () =>
    `<tooltip content="Deletes the branch — cannot be undone" placement="top">\n  <button tone="destructive" variant="outline">Delete branch</button>\n</tooltip>`

  return {
    placement,
    content,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    tipExTop,
    tipExSides,
    tipExDanger,
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
    <column slot="specimen" gap="3" align="center" style(specimen)>
      <tooltip content={content()} placement={placement()}>
        <span style(mutedSm)>ship.dev/d/ab12</span>
      </tooltip>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Placement</label>
        <select size="sm" bind:value={placement} label="Placement">
          <option value="top">top</option>
          <option value="bottom">bottom</option>
          <option value="left">left</option>
          <option value="right">right</option>
        </select>
      </column>
      <column gap="2">
        <label style(knobLabel)>Content</label>
        <select size="sm" bind:value={content} label="Content">
          <option value="Copy URL">Copy URL</option>
          <option value="Open in new tab">Open in new tab</option>
          <option value="Deletes the branch — cannot be undone">Destructive hint</option>
        </select>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="center" style(specimen)>
      <tooltip content="Copy URL" placement="top">
        <span style(mutedSm)>ship.dev/d/ab12</span>
      </tooltip>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Default top"
        lead="content attribute fills the bubble; never steals focus."
        code={tipExTop()}
      >
        <tooltip content="Copy URL" placement="top">
          <span style(mutedSm)>ship.dev/d/ab12</span>
        </tooltip>
      </doc-example>
      <doc-example
        title="Four sides"
        lead="placement top · bottom · left · right."
        code={tipExSides()}
      >
        <row gap="4" align="center" style(demoRow)>
          <tooltip content="Top" placement="top"><button size="sm" variant="outline">top</button></tooltip>
          <tooltip content="Bottom" placement="bottom"><button size="sm" variant="outline">bottom</button></tooltip>
          <tooltip content="Left" placement="left"><button size="sm" variant="outline">left</button></tooltip>
          <tooltip content="Right" placement="right"><button size="sm" variant="outline">right</button></tooltip>
        </row>
      </doc-example>
      <doc-example
        title="Destructive hint"
        lead="Longer content string on a danger control."
        code={tipExDanger()}
      >
        <tooltip content="Deletes the branch — cannot be undone" placement="top">
          <button tone="destructive" variant="outline" size="sm">Delete branch</button>
        </tooltip>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
