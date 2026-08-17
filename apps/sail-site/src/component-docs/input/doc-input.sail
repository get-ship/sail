<!--
  <doc-input> — /components/input detail host.
  Viewer owns split layout, snippet + copy, usage Code/Live/Split.
  Specimen is the bare control only — no absolute adornment hacks.
-->

<scope>
  style page = { size(max).overflow(visible) }
  style monoXs = { fg(muted).type(mono-sm) }
  style mutedSm = { fg(muted).type(body-sm) }
  /** Shared demo width + ring clearance. */
  style fieldShell = { size(measure).overflow(visible) }
  style fullW = { size(max).overflow(visible) }
  style exampleBlock = { size(max).overflow(visible) }
  style exampleSection = { size(max).overflow(visible) }
  style exampleH = { type(heading) }
  style exampleLead = { fg(muted).type(body-sm) }
  style sectionLabel = { fg(muted).type(label) }
  style mapRow = { size(max) }
  style growCol = { size(grow) }
</scope>

<controller>
import type { Scope } from './doc-input.sail'
import { DETAILS } from '../../compositions/component-details'

/** Empty text values so specimen, placeholder knob, and snippet stay aligned. */
const TYPE_DEFAULTS: Record<string, { placeholder: string; value: string }> = {
  text: { placeholder: 'Display name', value: '' },
  email: { placeholder: 'you@ship.dev', value: '' },
  password: { placeholder: 'Password', value: '' },
  number: { placeholder: '0', value: '' },
  search: { placeholder: 'Search…', value: '' },
  tel: { placeholder: '+1 (555) 010-0100', value: '' },
  url: { placeholder: 'https://ship.dev', value: '' },
  hidden: { placeholder: '', value: 'token' },
}

export default controller((_ctx: Scope) => {
  const INPUT_SIZES = ['xs', 'sm', 'md', 'lg'] as const
  const sizeIdx = signal(2)
  const inputType = signal('email')
  const inputInvalid = signal(false)
  const inputDisabled = signal(false)
  const emailDefaults = TYPE_DEFAULTS.email ?? TYPE_DEFAULTS.text ?? {
    placeholder: 'you@ship.dev',
    value: '',
  }
  const placeholder = signal(emailDefaults.placeholder)
  const textVal = signal(emailDefaults.value)

  const inputSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), INPUT_SIZES.length - 1)
    return INPUT_SIZES[i] ?? 'md'
  })
  const isHidden = derived(() => inputType() === 'hidden')
  const usesPlaceholder = derived(() => {
    const t = inputType()
    return t !== 'hidden'
  })
  const isTextField = derived(() => !isHidden())

  effect(() => {
    const t = inputType()
    const d = TYPE_DEFAULTS[t] ?? emailDefaults
    placeholder.set(d.placeholder)
    textVal.set(d.value)
  })

  const specimenCode = derived(() => {
    const t = inputType()
    const size = inputSize()
    const attrs: string[] = [`type="${t}"`]
    if (size !== 'md') attrs.push(`size="${size}"`)
    if (usesPlaceholder() && placeholder()) {
      attrs.push(`placeholder="${placeholder()}"`)
    }
    if (inputInvalid()) attrs.push('invalid')
    if (inputDisabled()) attrs.push('disabled')
    if (t === 'password') attrs.push('reveal')
    if (t === 'hidden') return `<input ${attrs.join(' ')} value="token" />`
    attrs.push('bind:value={value}')
    return `<input ${attrs.join(' ')} />`
  })

  const entry = DETAILS.input
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const exEmail = () =>
    `<input type="email" placeholder="you@ship.dev" bind:value={email} />`
  const exPassword = () =>
    `<input type="password" placeholder="Password" bind:value={secret} />`
  const exSearch = () =>
    `<input type="search" placeholder="Search…" bind:value={q} />`
  const exNumber = () =>
    `<input type="number" min="0" max="100" step="1" bind:value={qty} />`
  const exDates = () =>
    `<date type="date" />\n<date type="time" step="900" />\n<date type="datetime-local" />`
  const exFile = () => `<file />`
  const exColor = () => `<color value="#c8f542" />`
  const exInvalid = () =>
    `<input type="email" value="not-an-email" invalid />`
  const exSizes = () =>
    `<input size="xs" placeholder="xs" />\n<input size="sm" placeholder="sm" />\n<input placeholder="md" />\n<input size="lg" placeholder="lg" />`

  return {
    sizeIdx,
    inputType,
    inputInvalid,
    inputDisabled,
    placeholder,
    textVal,
    inputSize,
    isHidden,
    isTextField,
    usesPlaceholder,
    specimenCode,
    heading,
    blurb,
    props,
    usageCode,
    exEmail,
    exPassword,
    exSearch,
    exNumber,
    exDates,
    exFile,
    exColor,
    exInvalid,
    exSizes,
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
    <column slot="specimen" gap="3" align="center" style(fieldShell)>
      <if {isHidden()}>
        <p style(mutedSm)>Hidden fields take no space. They still submit with the form.</p>
      </if>
      <if {isTextField()}>
        <input
          type={inputType()}
          bind:value={textVal}
          placeholder={placeholder()}
          size={inputSize()}
          invalid={inputInvalid()}
          disabled={inputDisabled()}
          style(fullW)
 />
      </if>
    </column>

    <column slot="knobs" gap="5" align="stretch">
      <column gap="2">
        <label>Type</label>
        <select size="sm" bind:value={inputType} label="Type">
          <option value="text">text</option>
          <option value="email">email</option>
          <option value="password">password</option>
          <option value="number">number</option>
          <option value="search">search</option>
          <option value="tel">tel</option>
          <option value="url">url</option>
          <option value="hidden">hidden</option>
        </select>
      </column>
      <column gap="2">
        <label>Size</label>
        <slider bind:value={sizeIdx} min="0" max="3" step="1" size="sm" label="Size" ticks="xs,sm,md,lg"></slider>
      </column>
      <if {usesPlaceholder()}>
        <column gap="2">
          <label>Placeholder</label>
          <input type="text" bind:value={placeholder} size="sm" placeholder="Hint text" />
        </column>
      </if>
      <row gap="5" align="start">
        <column gap="2" style(growCol)>
          <label>Invalid</label>
          <switch bind:checked={inputInvalid} size="sm" label="Invalid"></switch>
        </column>
        <column gap="2" style(growCol)>
          <label>Disabled</label>
          <switch bind:checked={inputDisabled} size="sm" label="Disabled"></switch>
        </column>
      </row>
    </column>

    <column slot="usage-preview" gap="3" style(fieldShell)>
      <input type="email" placeholder="you@ship.dev" size="md" style(fullW) />
      <input type="email" value="not-an-email" style(fullW) invalid />
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Guide</p>

      <column gap="3" style(exampleSection)>
        <h3 style(exampleH)>Which control</h3>
        <p style(exampleLead)>
          Use this page for field values. Other shapes have their own elements — not more
          <code>type</code> values on <code>&lt;input&gt;</code>.
        </p>
        <column gap="2">
          <row gap="3" align="baseline" style(mapRow)>
            <code>&lt;input&gt;</code>
            <span style(mutedSm)>Text, email, password, search, number, tel, url, hidden</span>
          </row>
          <row gap="3" align="baseline" style(mapRow)>
            <code>&lt;checkbox&gt;</code>
            <span style(mutedSm)>
              Boolean check —
              <a href="/components/checkbox">checkbox</a>
            </span>
          </row>
          <row gap="3" align="baseline" style(mapRow)>
            <code>&lt;date&gt; · &lt;file&gt; · &lt;color&gt;</code>
            <span style(mutedSm)>
              Elevated pickers —
              <a href="/components/date">date</a> ·
              <a href="/components/file">file</a> ·
              <a href="/components/color">color</a>
            </span>
          </row>
          <row gap="3" align="baseline" style(mapRow)>
            <code>&lt;radio&gt;</code>
            <span style(mutedSm)>
              One option in a group — not bare <code>type="radio"</code>.
              <a href="/components/radio">radio</a> ·
              <a href="/components/radio-group">radio-group</a>
            </span>
          </row>
          <row gap="3" align="baseline" style(mapRow)>
            <code>&lt;slider&gt;</code>
            <span style(mutedSm)>Continuous value — <a href="/components/slider">slider</a></span>
          </row>
          <row gap="3" align="baseline" style(mapRow)>
            <code>&lt;switch&gt;</code>
            <span style(mutedSm)>On / off — <a href="/components/switch">switch</a></span>
          </row>
          <row gap="3" align="baseline" style(mapRow)>
            <code>&lt;select&gt; · &lt;combobox&gt;</code>
            <span style(mutedSm)>
              Lists —
              <a href="/components/select">select</a> ·
              <a href="/components/combobox">combobox</a>
            </span>
          </row>
          <row gap="3" align="baseline" style(mapRow)>
            <code>&lt;textarea&gt;</code>
            <span style(mutedSm)>Multi-line — <a href="/components/textarea">textarea</a></span>
          </row>
          <row gap="3" align="baseline" style(mapRow)>
            <code>&lt;button&gt;</code>
            <span style(mutedSm)>Actions and form submit — <a href="/components/button">button</a></span>
          </row>
          <row gap="3" align="baseline" style(mapRow)>
            <code>&lt;field&gt;</code>
            <span style(mutedSm)>Label, hint, and error around a control — <a href="/components/field">field</a></span>
          </row>
        </column>
      </column>

      <doc-example
        title="Email"
        lead='type="email" — mobile keyboards and basic validation come free.'
        code={exEmail()}
      >
        <column style(fieldShell)>
          <input type="email" placeholder="you@ship.dev" style(fullW) />
        </column>
      </doc-example>

      <doc-example
        title="Password"
        lead="Masked input with a reveal control by default. Pass reveal={false} to hide it."
        code={exPassword()}
      >
        <column style(fieldShell)>
          <input type="password" placeholder="Password" style(fullW) />
        </column>
      </doc-example>

      <doc-example
        title="Search"
        lead="Semantic search field. Some browsers add a clear control."
        code={exSearch()}
      >
        <column style(fieldShell)>
          <input type="search" placeholder="Search…" style(fullW) />
        </column>
      </doc-example>

      <doc-example
        title="Number"
        lead="Set min, max, and step as needed."
        code={exNumber()}
      >
        <column style(fieldShell)>
          <input type="number" min="0" max="100" step="1" placeholder="0" value="12" style(fullW) />
        </column>
      </doc-example>

      <doc-example
        title="Related controls"
        lead="Dates, files, and colours are their own components — not extra type values on input."
        code={exDates()}
      >
        <column gap="3">
          <card>
            <span style(monoXs)>&lt;date&gt;</span>
            <date type="date" style(fullW) />
          </card>
          <card>
            <span style(monoXs)>&lt;file&gt;</span>
            <file style(fullW) />
          </card>
          <card>
            <span style(monoXs)>&lt;color&gt;</span>
            <color value="#c8f542" />
          </card>
        </column>
      </doc-example>

      <doc-example
        title="Invalid"
        lead="Set invalid when validation fails. Pair with field for a message under the control."
        code={exInvalid()}
      >
        <column style(fieldShell)>
          <input type="email" value="not-an-email" style(fullW) invalid />
        </column>
      </doc-example>

      <doc-example
        title="Sizes"
        lead="Height from xs to lg. Default is md."
        code={exSizes()}
      >
        <column gap="2" style(fieldShell)>
          <input size="xs" placeholder="xs" style(fullW) />
          <input size="sm" placeholder="sm" style(fullW) />
          <input placeholder="md" style(fullW) />
          <input size="lg" placeholder="lg" style(fullW) />
        </column>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
