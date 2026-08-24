<!--
  <doc-drawer> — /components/drawer
  Shared chrome: split + snippet copy + usage Code|Live|Split.
  Specimen: command="show" invoker + <drawer> header/section/footer.
  command="show" is polyfilled by use:drawerSwipe (non-modal, not showModal).
-->

<scope>
  style page = { size({ w: 100%, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style mutedSm = { fg(muted-foreground).type(body-sm) }
  style mutedXs = { fg(muted-foreground).type(body-sm) }
  style specimen = { size({ w: 100%, max-w: panel-max-w-md }).pad(2).overflow({ all: visible }) }
  style knobLabel = { fg(muted-foreground).type(label) }
  style sizeValue = { fg(muted-foreground).type(mono-sm) }
  style exampleBlock = { size({ w: 100% }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style sectionLabel = { fg(muted-foreground).type(label) }
</scope>

<controller>
import type { Scope } from './doc-drawer.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const size = signal('md')
  const side = signal('end')
  // Not named `dismissible` — clashes with built-in use:dismissible action.
  const canDismiss = signal(true)

  const entry = DETAILS.drawer
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = ['id="filters"']
    if (size() !== 'md') a.push(`size="${size()}"`)
    if (side() !== 'end') a.push(`side="${side()}"`)
    if (!canDismiss()) a.push('dismissible={false}')
    return `<button command="show" commandfor="filters">Open drawer</button>\n<drawer ${a.join(' ')}>\n  <header>Filters</header>\n  <section>…</section>\n  <footer>\n    <button command="close" commandfor="filters" variant="ghost">Cancel</button>\n    <button command="close" commandfor="filters" tone="primary">Apply</button>\n  </footer>\n</drawer>`
  })

  const drExEnd = () =>
    `<button command="show" commandfor="filters">Filters</button>\n<drawer id="filters" side="end" size="md">\n  <header>Filters</header>\n  <section>…</section>\n  <footer>\n    <button command="close" commandfor="filters" variant="ghost">Cancel</button>\n    <button command="close" commandfor="filters" tone="primary">Apply</button>\n  </footer>\n</drawer>`
  const drExStart = () =>
    `<drawer side="start" size="sm">\n  <header>Nav</header>\n  <section>…</section>\n</drawer>`
  const drExLock = () =>
    `<drawer dismissible={false}>\n  <!-- ESC + swipe cannot close; user must pick an action -->\n</drawer>`

  return {
    size,
    side,
    canDismiss,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    drExEnd,
    drExStart,
    drExLock,
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
      <button size="sm" tone="primary" command="show" commandfor="doc-drawer-live">Open drawer</button>
      <drawer
        id="doc-drawer-live"
        size={size()}
        side={side()}
        dismissible={canDismiss() ? 'true' : 'false'}
      >
        <header>Filters</header>
        <section>
          <p style(mutedSm)>side · size knobs apply when opened. Header / section / footer anatomy. Swipe toward the edge closes.</p>
        </section>
        <footer>
          <button command="close" commandfor="doc-drawer-live" variant="ghost" size="sm">Cancel</button>
          <button command="close" commandfor="doc-drawer-live" variant="solid" tone="primary" size="sm">Apply</button>
        </footer>
      </drawer>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Side</label>
        <select size="sm" bind:value={side} label="Side">
          <option value="start">start</option>
          <option value="end">end</option>
          <option value="top">top</option>
          <option value="bottom">bottom</option>
        </select>
        <p style(mutedXs)>Edge the sheet docks to. Default is end.</p>
      </column>
      <column gap="2">
        <label style(knobLabel)>Size</label>
        <select size="sm" bind:value={size} label="Size">
          <option value="sm">sm</option>
          <option value="md">md</option>
          <option value="lg">lg</option>
        </select>
      </column>
      <column gap="2">
        <label style(knobLabel)>Dismissible</label>
        <row gap="2" align="center">
          <switch bind:checked={canDismiss} size="sm" tone="primary" label="Dismissible"></switch>
          <span style(sizeValue)>{canDismiss() ? 'ESC + swipe' : 'locked'}</span>
        </row>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="center" style(specimen)>
      <button size="sm" tone="primary" command="show" commandfor="doc-drawer-usage">Open drawer</button>
      <drawer id="doc-drawer-usage" size="md" side="end">
        <header>Filters</header>
        <section>
          <p style(mutedSm)>Header / section / footer anatomy.</p>
        </section>
        <footer>
          <button command="close" commandfor="doc-drawer-usage" variant="ghost" size="sm">Cancel</button>
          <button command="close" commandfor="doc-drawer-usage" variant="solid" tone="primary" size="sm">Apply</button>
        </footer>
      </drawer>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="End sheet"
        lead={'Native command="show" / close — non-modal, zero JS.'}
        code={drExEnd()}
      >
        <button size="sm" variant="outline" command="show" commandfor="doc-drawer-ex-end">Filters</button>
        <drawer id="doc-drawer-ex-end" side="end" size="md">
          <header>Filters</header>
          <section>
            <p style(mutedSm)>Sheet content</p>
          </section>
          <footer>
            <button command="close" commandfor="doc-drawer-ex-end" variant="ghost" size="sm">Cancel</button>
            <button command="close" commandfor="doc-drawer-ex-end" tone="primary" size="sm">Apply</button>
          </footer>
        </drawer>
      </doc-example>
      <doc-example
        title="Start rail"
        lead={'side="start" docks to the inline-start edge.'}
        code={drExStart()}
      >
        <button size="sm" variant="outline" command="show" commandfor="doc-drawer-ex-start">Open nav</button>
        <drawer id="doc-drawer-ex-start" side="start" size="sm">
          <header>Nav</header>
          <section>
            <p style(mutedSm)>Start-edge rail</p>
          </section>
          <footer>
            <button command="close" commandfor="doc-drawer-ex-start" variant="ghost" size="sm">Close</button>
          </footer>
        </drawer>
      </doc-example>
      <doc-example
        title="Locked dismiss"
        lead="dismissible=false — ESC and swipe will not close."
        code={drExLock()}
      >
        <button size="sm" variant="outline" command="show" commandfor="doc-drawer-ex-lock">Open locked</button>
        <drawer id="doc-drawer-ex-lock" dismissible="false">
          <header>Confirm</header>
          <section>
            <p style(mutedSm)>ESC and swipe will not close this drawer.</p>
          </section>
          <footer>
            <button command="close" commandfor="doc-drawer-ex-lock" variant="solid" tone="primary" size="sm">Acknowledge</button>
          </footer>
        </drawer>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components/dialog">Dialog</a>
  </row>
</column>
