<!--
  <theme-l1-inputs> — Themes → Components live stage: Inputs faces (button, field, select, …).
  Host bind: components={components}. One face visible via stage gates.
-->
<scope>
  style fieldHint = { paint: fg(muted-foreground), text: fontSize(xs) }
  style doorTitle = { text: fontWeight(semibold) }
  style iconBtn = { box: shrink(0).w(control-h-sm).h(control-h-sm).minW(control-h-sm).minH(control-h-sm) }
  style tapFloor = { box: shrink(0).minW(control-h-sm).minH(control-h-sm) }
  style stageHost = { box: w(full) }
  style sampleMax = { box: w(full).maxW(panel-max-w-sm) }
</scope>

<controller>
import type { UiEvent } from '../compositions/ui-event'

type StageBag = {
  stage: Record<string, (() => boolean) | undefined>
  variant: () => string
  sampleText: () => string
  onSampleInput: (e: UiEvent) => void
  load: () => number
  notify: () => boolean
  pageNo: () => string
  onPageChange: (detail: { page?: string }) => void
}

export default controller(() => {
  const bag = signal<StageBag | null>(null)
  mount((host) => {
    const pre = (host as { components?: StageBag | null }).components
    if (pre != null) bag.set(pre)
    Object.defineProperty(host, 'components', {
      configurable: true,
      get: () => bag(),
      set: (s: StageBag | null) => bag.set(s),
    })
  })
  const components = {
    get stage() {
      return new Proxy({} as Record<string, () => boolean>, {
        get(_t, prop: string) {
          if (typeof prop !== 'string') return () => false
          return bag()?.stage?.[prop] ?? (() => false)
        },
      })
    },
    variant: () => bag()?.variant() ?? '',
    sampleText: () => bag()?.sampleText() ?? '',
    onSampleInput: (e: UiEvent) => bag()?.onSampleInput(e),
    load: () => bag()?.load() ?? 40,
    notify: () => bag()?.notify() ?? false,
    pageNo: () => bag()?.pageNo() ?? '1',
    onPageChange: (d: { page?: string }) => bag()?.onPageChange(d),
  }
  return { components }
})
</controller>

<column gap="4" align="start" theme-l1-inputs l1-stage-host style(stageHost)>
<!-- ── Inputs ─────────────────────────────────────── -->
<if {components.stage.button()}>
  <column gap="4" style(stageHost)>
    <column gap="2">
      <p style(fieldHint)>Solid · tones</p>
      <row gap="2" align="center" nowrap>
        <button type="button" size="sm" variant="solid" tone="primary">Primary</button>
        <button type="button" size="sm" variant="solid" tone="success">Success</button>
        <button type="button" size="sm" variant="solid" tone="warning">Warning</button>
        <button type="button" size="sm" variant="solid" tone="destructive">Destructive</button>
        <button type="button" size="sm" variant="solid" tone="accent">Accent</button>
        <button type="button" size="iconSm" variant="solid" tone="primary" label="Settings" style(iconBtn)>
          <icon name="settings"></icon>
        </button>
      </row>
    </column>
    <column gap="2">
      <p style(fieldHint)>Soft · outline · ghost</p>
      <row gap="2" align="center" nowrap>
        <button type="button" size="sm" variant="soft" tone="primary">Soft</button>
        <button type="button" size="sm" variant="outline" tone="primary">Outline</button>
        <button type="button" size="sm" variant="ghost" tone="muted">Ghost</button>
        <button type="button" size="sm" variant={components.variant() || 'solid'} tone="primary">Active chip</button>
      </row>
    </column>
  </column>
</if>
<if {components.stage.a()}>
  <column gap="4" style(stageHost)>
    <column gap="2">
      <p style(fieldHint)>Bare text · + rest button-links (component.button.*)</p>
      <row gap="2" align="center" nowrap>
        <a href="#">text link</a>
        <a href="#" variant="default">default</a>
        <a href="#" variant="soft">soft</a>
        <a href="#" variant="outline">outline</a>
        <a href="#" variant="ghost">ghost</a>
      </row>
    </column>
    <column gap="2">
      <p style(fieldHint)>Toned button-links · active chip</p>
      <row gap="2" align="center" nowrap>
        <a href="#" variant={components.variant() || 'solid'} tone="primary">Primary</a>
        <a href="#" variant="solid" tone="success">Success</a>
        <a href="#" variant="outline" tone="warning">Warn</a>
        <a href="#" variant="ghost" tone="destructive">Danger</a>
        <a href="#" variant="soft" tone="accent">Accent</a>
      </row>
    </column>
  </column>
</if>
<if {components.stage.input()}>
  <column gap="3" style(sampleMax)>
    <column gap="2">
      <p style(fieldHint)>Text field · sizes · component.field.bg/fg/border</p>
      <input type="text" size="sm" placeholder="sm field" value={components.sampleText()} on:input={components.onSampleInput} label="Sample field" />
      <input type="text" size="md" placeholder="md field" />
      <input type="text" size="lg" placeholder="lg field" />
    </column>
    <column gap="2">
      <p style(fieldHint)>Invalid · disabled</p>
      <input type="email" size="sm" value="not-an-email" invalid />
      <input type="text" size="sm" placeholder="Disabled" disabled />
    </column>
    <column gap="2">
      <p style(fieldHint)>Checkbox type (not a separate tag) · range</p>
      <row gap="3" align="center">
        <input type="checkbox" checked />
        <input type="checkbox" />
        <input type="checkbox" disabled />
        <input type="checkbox" size="lg" checked />
      </row>
      <input type="range" min="0" max="100" value="40" />
    </column>
  </column>
</if>
<if {components.stage.textarea()}>
  <column gap="3" style(sampleMax)>
    <column gap="2">
      <p style(fieldHint)>Sizes · component.field.* surface</p>
      <textarea size="xs" rows="2" placeholder="xs"></textarea>
      <textarea size="sm" rows="2" placeholder="sm"></textarea>
      <textarea size={components.variant() || 'md'} rows="3" placeholder="md (chip or default)"></textarea>
      <textarea size="lg" rows="2" placeholder="lg"></textarea>
    </column>
    <column gap="2">
      <p style(fieldHint)>Invalid · disabled</p>
      <textarea invalid rows="2" placeholder="invalid → destructive border"></textarea>
      <textarea disabled rows="2" placeholder="disabled"></textarea>
    </column>
  </column>
</if>
<if {components.stage.select()}>
  <column gap="3" style(sampleMax)>
    <column gap="2">
      <p style(fieldHint)>Sizes · component.field surface + SELECT_MAP chevron</p>
      <select size="xs">
        <option>xs</option>
        <option>Engineering</option>
      </select>
      <select size="sm">
        <option>sm</option>
        <option>Engineering</option>
      </select>
      <select size={components.variant() || 'md'}>
        <option>md</option>
        <option>Design</option>
      </select>
      <select size="lg">
        <option>lg</option>
        <option>Research</option>
      </select>
    </column>
    <column gap="2">
      <p style(fieldHint)>Rest / invalid / disabled</p>
      <select>
        <option>Rest field tokens</option>
        <option>Design</option>
      </select>
      <select invalid>
        <option>Invalid border</option>
        <option>Engineering</option>
      </select>
      <select disabled>
        <option>Disabled</option>
      </select>
    </column>
    <p style(fieldHint)>
      component.field bg/fg/border · hover free border-strong · not combobox
    </p>
  </column>
</if>
<if {components.stage.combobox()}>
  <combobox placeholder="Type to filter…" style(sampleMax)>
    <option value="sail">Sail</option>
    <option value="react">React</option>
    <option value="svelte">Svelte</option>
  </combobox>
</if>
<if {components.stage.radio()}>
  <column gap="2" style(sampleMax)>
    <radio value="a">Option A</radio>
    <radio value="b" disabled>Disabled option</radio>
    <p style(fieldHint)>component.radio.fg · value + disabled only</p>
  </column>
</if>
<if {components.stage.radioGroup()}>
  <radio-group label="Plan" name="theme-l1-plan" value="monthly" style(sampleMax)>
    <radio value="monthly">Monthly</radio>
    <radio value="yearly">Yearly</radio>
  </radio-group>
</if>
<if {components.stage.switchToggle()}>
  <column gap="3" style(sampleMax)>
    <column gap="2">
      <p style(fieldHint)>Sizes · component.switch.track + thumb</p>
      <row gap="3" align="center">
        <switch size="sm" checked></switch>
        <switch size={components.variant() || 'md'} checked></switch>
        <switch size="lg" checked></switch>
      </row>
    </column>
    <column gap="2">
      <p style(fieldHint)>Off / on / disabled (rest leaves)</p>
      <row gap="3" align="center">
        <switch></switch>
        <switch checked={components.notify()}></switch>
        <switch disabled></switch>
        <switch checked disabled></switch>
      </row>
    </column>
    <column gap="2">
      <p style(fieldHint)>Tone · checked track only; thumb stays switch.thumb</p>
      <row gap="3" align="center">
        <switch tone="primary" checked></switch>
        <switch tone="success" checked></switch>
        <switch tone="warning" checked></switch>
        <switch tone="destructive" checked></switch>
      </row>
    </column>
  </column>
</if>
<if {components.stage.slider()}>
  <column gap="3" style(sampleMax)>
    <column gap="2">
      <p style(fieldHint)>Sizes · component.slider.track + fill + track-border</p>
      <slider value="40" min="0" max="100" size="sm"></slider>
      <slider value={components.load()} min="0" max="100" size={components.variant() || 'md'}></slider>
      <slider value="80" min="0" max="100" size="lg"></slider>
    </column>
    <column gap="2">
      <p style(fieldHint)>Disabled · native freeze + opacity</p>
      <slider value="55" min="0" max="100" disabled></slider>
    </column>
    <p style(fieldHint)>Thumb face = field · rim = slider.fill · single thumb only</p>
  </column>
</if>
<if {components.stage.toggle() || components.stage.toggleGroup()}>
  <!-- standalone toggle died; pressed-button lives only in toggle-group -->
  <toggle-group variant={components.variant() || 'outline'}>
    <toggle value="left">Left</toggle>
    <toggle value="center">Center</toggle>
    <toggle value="right">Right</toggle>
  </toggle-group>
</if>
<if {components.stage.field()}>
  <field style(sampleMax)>
    <label slot="label" for="theme-l1-field">Name</label>
    <input id="theme-l1-field" size="sm" placeholder="Ada Lovelace" />
    <p slot="hint">Hint under the control</p>
  </field>
</if>
<if {components.stage.label()}>
  <column gap="3" style(sampleMax)>
    <column gap="2">
      <p style(fieldHint)>Rest · sizes · component.label.fg + weight</p>
      <label for="theme-l1-label-sm" size="sm">Email sm</label>
      <label for="theme-l1-label-md" size={components.variant() || 'md'}>Email md</label>
      <label for="theme-l1-label-lg" size="lg">Email lg</label>
    </column>
    <column gap="2">
      <p style(fieldHint)>Required · component.label.required marker</p>
      <label for="theme-l1-label" size={components.variant() || 'md'} required>Email</label>
      <input id="theme-l1-label" size="sm" placeholder="ada@example.com" />
    </column>
    <column gap="1">
      <p style(fieldHint)>Optional (no marker)</p>
      <label for="theme-l1-label-opt" size="sm">Nickname</label>
      <input id="theme-l1-label-opt" size="sm" placeholder="optional" />
    </column>
  </column>
</if>
<if {components.stage.fieldset()}>
  <column gap="3" style(sampleMax)>
    <column gap="2">
      <p style(fieldHint)>Rest · component.fieldset.border</p>
      <fieldset style(sampleMax)>
        <legend>Account</legend>
        <column gap="2">
          <input size="sm" placeholder="Name" />
          <input size="sm" placeholder="Email" />
        </column>
      </fieldset>
    </column>
    <column gap="2">
      <p style(fieldHint)>Disabled · native freeze + opacity</p>
      <fieldset disabled style(sampleMax)>
        <legend>Billing (locked)</legend>
        <column gap="2">
          <input size="sm" placeholder="Card" />
          <button type="button" variant="solid" size="sm">Save</button>
        </column>
      </fieldset>
    </column>
  </column>
</if>
<if {components.stage.legend()}>
  <column gap="3" style(sampleMax)>
    <p style(fieldHint)>Rest · component.legend.fg (theme override)</p>
    <fieldset>
      <legend>Account details</legend>
      <column gap="2">
        <input size="sm" placeholder="Name" />
        <input size="sm" placeholder="Email" />
      </column>
    </fieldset>
    <fieldset>
      <legend>Billing options</legend>
      <p style(fieldHint)>Caption only — nest inside fieldset.</p>
    </fieldset>
  </column>
</if>
<if {components.stage.themePicker()}>
  <theme-picker></theme-picker>
</if>
</column>
