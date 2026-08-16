<!--
  <theme-l1-feedback> — Themes → Components live stage: Feedback faces (alert, toast, spinner, …).
  Host bind: components={components}. One face visible via stage gates.
-->
<scope>
  style fieldHint = { fg(muted-foreground).type({ size: xs }) }
  style doorTitle = { type({ weight: semibold }) }
  style iconBtn = { shrink(0).size({ w: control-h-sm, h: control-h-sm, min-w: control-h-sm, min-h: control-h-sm }) }
  style tapFloor = { shrink(0).size({ min-w: control-h-sm, min-h: control-h-sm }) }
  style stageHost = { size({ w: full }) }
  style sampleMax = { size({ w: full, max-w: panel-max-w-sm }) }
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

<column gap="4" align="start" theme-l1-feedback l1-stage-host style(stageHost)>
<if {components.stage.alert()}>
  <column gap="3" style(stageHost)>
    <column gap="2">
      <p style(fieldHint)>Rest · tone roles (info / success / warning / danger)</p>
      <alert style(stageHost)>
        <strong slot="title">Notice</strong>
        Rest paint uses theme tone roles.
      </alert>
    </column>
    <column gap="2">
      <p style(fieldHint)>Toned · stripe + icon brand (host fill stays theme roles)</p>
      <column gap="2">
        <alert tone="info" style(stageHost)>
          <strong slot="title">Heads up</strong>
          Info stripe under theme paint.
        </alert>
        <alert tone="success" style(stageHost)>
          <strong slot="title">Saved</strong>
          Success stripe under theme paint.
        </alert>
        <alert tone="warning" style(stageHost)>
          <strong slot="title">Check input</strong>
          Warning is assertive liveRegion.
        </alert>
        <alert tone="destructive" style(stageHost)>
          <strong slot="title">Failed</strong>
          Destructive is assertive liveRegion.
        </alert>
        <alert tone="muted" style(stageHost)>
          <strong slot="title">Note</strong>
          Muted keeps alert.stripe honesty.
        </alert>
      </column>
    </column>
  </column>
</if>
<if {components.stage.progressBar()}>
  <column gap="3" style(sampleMax)>
    <column gap="2">
      <p style(fieldHint)>Track/fill · muted + primary</p>
      <progress value="30" max="100" size="sm"></progress>
      <progress value={components.load()} max="100"></progress>
      <progress value="90" max="100" size="lg"></progress>
    </column>
    <column gap="2">
      <p style(fieldHint)>Indeterminate · omit value (native + PROGRESS_MAP)</p>
      <progress max="100"></progress>
    </column>
  </column>
</if>
<if {components.stage.meter()}>
  <column gap="3" style(sampleMax)>
    <column gap="2">
      <p style(fieldHint)>Rest · sizes · muted track + semantic bands</p>
      <meter value="0.55" size="sm"></meter>
      <meter value="0.55"></meter>
      <meter value="0.55" size="lg"></meter>
    </column>
    <column gap="2">
      <p style(fieldHint)>Bands · low=33 high=66 optimum=100</p>
      <meter value="20" min="0" max="100" low="33" high="66" optimum="100"></meter>
      <meter value="50" min="0" max="100" low="33" high="66" optimum="100"></meter>
      <meter value="85" min="0" max="100" low="33" high="66" optimum="100"></meter>
    </column>
  </column>
</if>
<if {components.stage.spinner()}>
  <column gap="3" style(stageHost)>
    <column gap="2">
      <p style(fieldHint)>Rest · muted arc + track</p>
      <row gap="4" align="center" nowrap>
        <spinner size="sm"></spinner>
        <spinner></spinner>
        <spinner size="lg"></spinner>
      </row>
    </column>
    <column gap="2">
      <p style(fieldHint)>Toned · size ladder + host tones</p>
      <row gap="4" align="center" nowrap>
        <spinner size="xs" tone="primary"></spinner>
        <spinner size="sm" tone="info"></spinner>
        <spinner size={components.variant() || 'md'} tone="success"></spinner>
        <spinner size="lg" tone="destructive"></spinner>
        <spinner tone="accent"></spinner>
      </row>
    </column>
  </column>
</if>
<if {components.stage.skeleton()}>
  <column gap="3" style(sampleMax)>
    <column gap="2">
      <p style(fieldHint)>Sheen · muted fill</p>
      <column gap="2">
        <skeleton shape="text" size="md"></skeleton>
        <skeleton shape="text" size="sm"></skeleton>
        <skeleton shape="line" size="sm"></skeleton>
      </column>
    </column>
    <column gap="2">
      <p style(fieldHint)>Shapes · circle + block</p>
      <row gap="3" align="center">
        <skeleton shape="circle" size="md"></skeleton>
        <skeleton shape="circle" size="sm"></skeleton>
        <skeleton shape="block" style({ size({ w: 24, min-h: 16 }) })></skeleton>
      </row>
    </column>
  </column>
</if>
<if {components.stage.emptyState()}>
  <column gap="3" style(sampleMax)>
    <column gap="2">
      <p style(fieldHint)>Rest · muted surface + border + title</p>
      <empty-state size="sm" style(stageHost)>
        <icon slot="icon" name="inbox"></icon>
        <strong slot="title">Empty inbox</strong>
        Messages you receive land here.
      </empty-state>
    </column>
    <column gap="2">
      <p style(fieldHint)>Inline · dashed chrome + tone on icon</p>
      <empty-state variant="inline" size="sm" tone="primary" style(stageHost)>
        <icon slot="icon" name="inbox"></icon>
        <strong slot="title">No results</strong>
        Try another filter.
      </empty-state>
    </column>
    <column gap="2">
      <p style(fieldHint)>Connecting · bare + status live region</p>
      <empty-state variant="connecting" size="sm" style(stageHost)>
        <spinner slot="icon" size="sm"></spinner>
        <strong slot="title">Connecting…</strong>
        Establishing a secure session.
      </empty-state>
    </column>
  </column>
</if>
<if {components.stage.errorState()}>
  <column gap="4" style(stageHost)>
    <column gap="2">
      <p style(fieldHint)>Rest · muted surface + border</p>
      <error-state code="ERR_THEME" size="sm">
        <strong slot="title">Couldn't load preview</strong>
        Rest chrome — muted surface + destructive icon.
        <button slot="action" type="button" variant="outline" size="sm">Retry</button>
      </error-state>
    </column>
    <column gap="2">
      <p style(fieldHint)>Inline · brand-destructive border</p>
      <error-state variant="inline" size="sm" code="ERR_INLINE">
        Inline recovery row on muted fill.
        <button slot="action" type="button" variant="ghost" size="sm">Retry</button>
      </error-state>
    </column>
    <column gap="2">
      <p style(fieldHint)>Details + code chip</p>
      <error-state size="sm" code="ERR_STACK">
        <strong slot="title">Upstream failed</strong>
        Stack collapses behind Show details.
        <span slot="details">at fetchData (api.ts:42)
  Error: gateway timeout</span>
        <button slot="action" type="button" variant="solid" tone="destructive" size="sm">Retry</button>
      </error-state>
    </column>
  </column>
</if>
<if {components.stage.errorBoundary()}>
  <error-boundary message="Panel failed to load.">
    <p style(fieldHint)>Healthy boundary is invisible — this is the wrapped tree.</p>
  </error-boundary>
</if>
<if {components.stage.toast()}>
  <toast tone="success" duration="0" style(sampleMax)>
    <strong>Saved.</strong> Sticky toast (duration 0).
  </toast>
</if>
<if {components.stage.toastRegion()}>
  <column gap="2" style({ size({ w: full, min-h: 24, max-w: panel-max-w-sm }).position(relative) })>
    <p style(fieldHint)>Toast region hosts stacked toasts (fixed in product).</p>
    <toast tone="info" duration="0">Region child toast</toast>
  </column>
</if>
</column>
