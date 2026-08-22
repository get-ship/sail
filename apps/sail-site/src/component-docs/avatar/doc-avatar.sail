<!--
  <doc-avatar> — /components/avatar
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { size({ w: 100%, max-w: breakpoint-desktop }).margin({ x: auto }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style sizeValue = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style tickRow = { size({ w: 100% }).pad({ x: 1_5, t: 1 }) }
  style tick = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style swatch = { size(7).shrink(0).radius(full).cursor(pointer).border({ color: transparent, width: 2 }).elevation(xs), when: { selected: border({ color: foreground }).elevation(sm), hover: filter({ opacity: 90 }) } }
  style swatchPrimary = { bg(primary) }
  style swatchInfo = { bg(info) }
  style swatchSuccess = { bg(success) }
  style swatchWarning = { bg(warning) }
  style swatchDanger = { bg(danger) }
  style swatchAccent = { bg(accent) }
  style swatchMuted = { bg(muted).border({ color: border }) }
  style exampleBlock = { size({ w: 100% }).pad({ t: 2, b: 4 }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
  style demoRow = { size({ w: 100% }).pad({ y: 2 }) }
  style mutedXs = { fg(muted-foreground).type({ size: xs }) }
</scope>

<controller>
import type { Scope } from './doc-avatar.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const AV_SIZES = ['xs', 'sm', 'md', 'lg', 'xl'] as const
  const sizeIdx = signal(2)
  const shape = signal('round')
  const avTone = signal('muted')
  const name = signal('Ada Lovelace')
  const initials = signal('')
  const pickTone = (t: string): void => avTone.set(t)

  const size = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), AV_SIZES.length - 1)
    return AV_SIZES[i] ?? 'md'
  })

  const entry = DETAILS.avatar
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = []
    if (size() !== 'md') a.push(`size="${size()}"`)
    if (shape() !== 'round') a.push(`shape="${shape()}"`)
    a.push(`tone="${avTone()}"`)
    if (initials()) {
      a.push(`initials="${initials()}"`)
    } else {
      a.push(`name="${name()}"`)
    }
    return `<avatar ${a.join(' ')}></avatar>`
  })

  const avExName = () =>
    `<avatar name="Ada Lovelace"></avatar>\n<avatar initials="GH" tone="primary"></avatar>`
  const avExShape = () =>
    `<avatar name="Alan Turing" size="lg" shape="square"></avatar>`
  const avExSizes = () =>
    `<avatar size="xs" initials="XS"></avatar>\n<avatar size="sm" initials="SM"></avatar>\n<avatar size="md" initials="MD"></avatar>\n<avatar size="lg" initials="LG"></avatar>\n<avatar size="xl" initials="XL"></avatar>`

  return {
    sizeIdx,
    size,
    shape,
    avTone,
    name,
    initials,
    pickTone,
    specimenCode,
    heading,
    blurb,
    props,
    usageCode,
    avExName,
    avExShape,
    avExSizes,
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
    <column slot="specimen" gap="4" align="center">
      <if {initials()}>
        <avatar size={size()} shape={shape()} tone={avTone()} initials={initials()}></avatar>
      {else =>
        <avatar size={size()} shape={shape()} tone={avTone()} name={name()}></avatar>}</if>
    </column>

    <column slot="knobs" gap="5" align="stretch">
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <label style(knobLabel)>Size</label>
          <span style(sizeValue)>{size()}</span>
        </row>
        <slider bind:value={sizeIdx} min="0" max="4" step="1" size="sm" label="Size" ticks="xs,sm,md,lg,xl"></slider>
      </column>
      <column gap="2">
        <label style(knobLabel)>Shape</label>
        <select size="sm" bind:value={shape} label="Shape">
          <option value="round">round</option>
          <option value="square">square</option>
        </select>
      </column>
      <column gap="2">
        <label style(knobLabel)>Tone</label>
        <row gap="2" align="center">
          <div
            title="primary"
            selected={avTone() === 'primary' ? 'true' : null}
            on:click={() => pickTone('primary')}
            style(swatch, swatchPrimary)
          ></div>
          <div
            title="accent"
            selected={avTone() === 'accent' ? 'true' : null}
            on:click={() => pickTone('accent')}
            style(swatch, swatchAccent)
          ></div>
          <div
            title="info"
            selected={avTone() === 'info' ? 'true' : null}
            on:click={() => pickTone('info')}
            style(swatch, swatchInfo)
          ></div>
          <div
            title="success"
            selected={avTone() === 'success' ? 'true' : null}
            on:click={() => pickTone('success')}
            style(swatch, swatchSuccess)
          ></div>
          <div
            title="warning"
            selected={avTone() === 'warning' ? 'true' : null}
            on:click={() => pickTone('warning')}
            style(swatch, swatchWarning)
          ></div>
          <div
            title="danger"
            selected={avTone() === 'danger' ? 'true' : null}
            on:click={() => pickTone('danger')}
            style(swatch, swatchDanger)
          ></div>
          <div
            title="muted"
            selected={avTone() === 'muted' ? 'true' : null}
            on:click={() => pickTone('muted')}
            style(swatch, swatchMuted)
          ></div>
        </row>
      </column>
      <column gap="2">
        <label style(knobLabel)>Name</label>
        <select size="sm" bind:value={name} label="Name">
          <option value="Ada Lovelace">Ada Lovelace</option>
          <option value="Grace Hopper">Grace Hopper</option>
          <option value="Alan Turing">Alan Turing</option>
        </select>
        <p style(mutedXs)>Fallback initials from name when initials is empty.</p>
      </column>
      <column gap="2">
        <label style(knobLabel)>Initials</label>
        <select size="sm" bind:value={initials} label="Initials">
          <option value="">(from name)</option>
          <option value="AL">AL</option>
          <option value="GH">GH</option>
          <option value="SK">SK</option>
        </select>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="center">
      <avatar name="Ada Lovelace"></avatar>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <doc-example
        title="Name + initials"
        lead="Broken images fall back to initials."
        code={avExName()}
      >
        <row gap="2" align="center" style(demoRow)>
          <avatar name="Ada Lovelace"></avatar>
          <avatar initials="GH" tone="primary"></avatar>
        </row>
      </doc-example>

      <doc-example
        title="Square large"
        lead='shape="square" with size="lg".'
        code={avExShape()}
      >
        <row gap="2" align="center" style(demoRow)>
          <avatar name="Alan Turing" size="lg" shape="square"></avatar>
        </row>
      </doc-example>

      <doc-example
        title="Size ladder"
        lead="xs · sm · md · lg · xl."
        code={avExSizes()}
      >
        <row gap="2" align="center" style(demoRow)>
          <avatar size="xs" initials="XS"></avatar>
          <avatar size="sm" initials="SM"></avatar>
          <avatar size="md" initials="MD"></avatar>
          <avatar size="lg" initials="LG"></avatar>
          <avatar size="xl" initials="XL"></avatar>
        </row>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
