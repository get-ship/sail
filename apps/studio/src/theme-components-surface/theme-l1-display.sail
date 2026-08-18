<!--
  <theme-l1-display> — Themes → Components live stage: Display faces (badge, chip, card, …).
  Host bind: components={components}. One face visible via stage gates.
-->
<scope>
  style fieldHint = { fg(muted).type(body-sm) }
  // no preset: type weight
  style doorTitle = { type({ weight: semibold }) }
  style iconBtn = { size(control-sm) }
  style tapFloor = { size(control-sm) }
  style stageHost = { size(max) }
  // no preset: size lock
  style sampleMax = { size({ w: 100%, max-w: panel-max-w-sm }) }
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

<column gap="4" align="start" theme-l1-display l1-stage-host style(stageHost)>
<if {components.stage.badge()}>
  <column gap="3" style(stageHost)>
    <column gap="2">
      <p style(fieldHint)>Rest · primary / muted tone roles</p>
      <row gap="2" align="center" nowrap>
        <badge variant="soft" size="sm">soft</badge>
        <badge variant="solid" size="sm">solid</badge>
        <badge variant="outline" size="sm">outline</badge>
      </row>
    </column>
    <column gap="2">
      <p style(fieldHint)>Toned · soft + solid + pill</p>
      <row gap="2" align="center" nowrap>
        <badge variant="soft" tone="primary" size="sm">primary</badge>
        <badge variant="soft" tone="success" size="sm">success</badge>
        <badge variant="solid" tone="warning" size="sm" pill>warn</badge>
        <badge variant="outline" tone="destructive" size="sm">err</badge>
        <badge variant="soft" tone="accent" size="sm" pill>12</badge>
      </row>
    </column>
  </column>
</if>
<if {components.stage.chip()}>
  <column gap="3" style(stageHost)>
    <column gap="2">
      <p style(fieldHint)>Rest · primary / muted / border</p>
      <row gap="2" align="center" nowrap>
        <chip variant="solid" size="sm">solid</chip>
        <chip variant="soft" size="sm">soft</chip>
        <chip variant="outline" size="sm">outline</chip>
        <chip variant="ghost" size="sm">ghost</chip>
      </row>
    </column>
    <column gap="2">
      <p style(fieldHint)>Toned · selected · removable</p>
      <row gap="2" align="center" nowrap>
        <chip variant={components.variant() || 'solid'} tone="primary" size="sm" selected>Selected</chip>
        <chip variant="soft" tone="success" size="sm">success</chip>
        <chip variant="outline" tone="warning" size="sm">warn</chip>
        <chip variant="solid" tone="destructive" size="sm" removable>err</chip>
        <chip variant="soft" tone="accent" size="sm">12</chip>
      </row>
    </column>
  </column>
</if>
<if {components.stage.card()}>
  <card variant={components.variant() || 'default'} padding="sm" style(sampleMax)>
    <column gap="2">
      <p style(doorTitle)>Card surface</p>
      <p style(fieldHint)>Judges the radius role when overridden.</p>
    </column>
  </card>
</if>
<if {components.stage.avatar()}>
  <row gap="3" align="center" nowrap>
    <avatar initials="AL" size={components.variant() || 'md'} tone="primary" label="Ada"></avatar>
    <avatar initials="GH" size={components.variant() || 'md'} shape="square" label="GH"></avatar>
  </row>
</if>
<if {components.stage.icon()}>
  <column gap="3" style(stageHost)>
    <column gap="2">
      <p style(fieldHint)>Size ladder · theme icon scale (xs–xl)</p>
      <row gap="4" align="center" nowrap>
        <icon name="sparkles" size="xs" label="xs"></icon>
        <icon name="sparkles" size="sm" label="sm"></icon>
        <icon name="sparkles" size={components.variant() || 'md'} label="md"></icon>
        <icon name="sparkles" size="lg" label="lg"></icon>
        <icon name="sparkles" size="xl" label="xl"></icon>
      </row>
    </column>
    <column gap="2">
      <p style(fieldHint)>Spin · use:icon motion · currentColor ink</p>
      <row gap="4" align="center" nowrap>
        <icon name="refresh" spin label="Refreshing"></icon>
        <span style({ fg(primary) })>
          <icon name="heart" size="lg" label="primary ink"></icon>
        </span>
        <span style({ fg(destructive) })>
          <icon name="heart" size="lg" label="destructive ink"></icon>
        </span>
      </row>
    </column>
  </column>
</if>
<if {components.stage.separator()}>
  <column gap="3" style(sampleMax)>
    <column gap="2">
      <p style(fieldHint)>Rest · border role</p>
      <separator></separator>
      <separator variant="dashed"></separator>
    </column>
    <column gap="2">
      <p style(fieldHint)>Vertical · same face between labels</p>
      <!-- no preset: size lock -->
      <row gap="3" align="center" style({ size({ h: 10 }) })>
        <span style(fieldHint)>A</span>
        <separator orientation="vertical"></separator>
        <span style(fieldHint)>B</span>
        <separator orientation="vertical" variant="dashed"></separator>
        <span style(fieldHint)>C</span>
      </row>
    </column>
    <column gap="2">
      <p style(fieldHint)>Toned · solid + dashed (free tones, not rest leaf)</p>
      <separator tone="primary"></separator>
      <separator variant="dashed" tone="destructive"></separator>
    </column>
  </column>
</if>
<if {components.stage.accordion()}>
  <accordion style(sampleMax)>
    <strong slot="trigger">Show details</strong>
    <p style(fieldHint)>Accordion body content.</p>
  </accordion>
</if>
<if {components.stage.kbd()}>
  <column gap="3" style(sampleMax)>
    <column gap="2">
      <p style(fieldHint)>Rest · muted fill + foreground + border</p>
      <row gap="2" align="center">
        <kbd>Ctrl</kbd>
        <span style(fieldHint)>+</span>
        <kbd>K</kbd>
      </row>
    </column>
    <column gap="2">
      <p style(fieldHint)>Modifiers · mono keycap chrome</p>
      <row gap="2" align="center">
        <kbd>⌘</kbd>
        <kbd>⇧</kbd>
        <kbd>⌥</kbd>
        <kbd>⌃</kbd>
        <kbd>Esc</kbd>
        <kbd>F12</kbd>
      </row>
    </column>
    <p style(fieldHint)>
      Press <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>P</kbd> in docs.
    </p>
  </column>
</if>
<if {components.stage.prose()}>
  <column gap="3" style(sampleMax)>
    <p style(fieldHint)>Rest · foreground</p>
    <prose>
      <h2>Typography scope</h2>
      <p>
        Body copy with <strong>strong</strong>, <em>emphasis</em>,
        <code>inline()</code>, and <mark>mark</mark>.
      </p>
      <blockquote>A quotation carries its own rule.</blockquote>
      <ul>
        <li>List item one</li>
        <li>List item two</li>
      </ul>
    </prose>
  </column>
</if>
<if {components.stage.codeBlock()}>
  <column gap="3" style(sampleMax)>
    <p style(fieldHint)>Rest · surface fill + foreground</p>
    <code-block
      code={'const n = signal(0)\nconst double = derived(() => n() * 2)\n// .tok-* ink via --syntax-*'}
      lang="ts"
    ></code-block>
    <code-block
      code={'<button variant="solid" tone="primary">\n  Save\n</button>'}
      lang="sail"
    ></code-block>
  </column>
</if>
</column>
