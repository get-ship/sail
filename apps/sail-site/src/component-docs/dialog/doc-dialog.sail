<!--
  <doc-dialog> — /components/dialog
  Shared chrome: split + snippet copy + usage Code|Live|Split.
  Specimen: invoker opens the surface; knobs for size · backdrop · side · dismissible.
-->

<scope>
  style page = { box: size({ w: full, max-w: breakpoint-desktop }).space({ mx: auto }).overflow({ all: visible }) }
  style mutedSm = { paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
  style mutedXs = { paint: ink({ color: muted-foreground }), text: type({ size: xs }) }
  style specimen = { box: size({ w: full, max-w: panel-max-w-md }).space({ pad: 2 }).overflow({ all: visible }) }
  style knobLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, weight: medium, tracking: wide, case: uppercase }),
}
  style sizeValue = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style exampleBlock = { box: size({ w: full }).space({ pad-t: 2, pad-b: 4 }).overflow({ all: visible }) }
  style sectionLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, tracking: wide, case: uppercase }),
}
</scope>

<controller>
import type { Scope } from './doc-dialog.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const size = signal('md')
  const backdrop = signal('default')
  const side = signal('center')
  // Not named `dismissible` — clashes with built-in use:dismissible action.
  const canDismiss = signal(true)

  const entry = DETAILS.dialog
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = ['id="confirm"']
    if (size() !== 'md') a.push(`size="${size()}"`)
    if (backdrop() !== 'default') a.push(`backdrop="${backdrop()}"`)
    if (side() !== 'center') a.push(`side="${side()}"`)
    if (!canDismiss()) a.push('dismissible={false}')
    return `<button command="show-modal" commandfor="confirm">Open dialog</button>\n<dialog ${a.join(' ')}>\n  <header>Invite to workspace</header>\n  <section>…</section>\n  <footer>\n    <button command="close" commandfor="confirm" variant="ghost">Cancel</button>\n    <button command="close" commandfor="confirm" tone="primary">Send</button>\n  </footer>\n</dialog>`
  })

  const dlgExModal = () =>
    `<button command="show-modal" commandfor="confirm">Delete…</button>\n<dialog id="confirm" size="md">\n  <header>Delete project</header>\n  <section>This cannot be undone.</section>\n  <footer>\n    <button command="close" commandfor="confirm" variant="ghost">Cancel</button>\n    <button command="close" commandfor="confirm" tone="destructive">Delete</button>\n  </footer>\n</dialog>`
  const dlgExSheet = () =>
    `<dialog side="end" size="sm">\n  <header>Filters</header>\n  <section>…</section>\n  <footer>…</footer>\n</dialog>`
  const dlgExLock = () =>
    `<dialog dismissible={false}>\n  <!-- ESC + backdrop cannot close; user must pick an action -->\n</dialog>`

  return {
    size,
    backdrop,
    side,
    canDismiss,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    dlgExModal,
    dlgExSheet,
    dlgExLock,
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
      <button size="sm" tone="primary" command="show-modal" commandfor="doc-dlg-live">Open dialog</button>
      <dialog
        id="doc-dlg-live"
        size={size()}
        backdrop={backdrop()}
        side={side()}
        dismissible={canDismiss() ? 'true' : 'false'}
      >
        <header>Invite to workspace</header>
        <section>
          <p style(mutedSm)>size · backdrop · side knobs apply when opened. Header / section / footer anatomy.</p>
        </section>
        <footer>
          <button command="close" commandfor="doc-dlg-live" variant="ghost" size="sm">Cancel</button>
          <button command="close" commandfor="doc-dlg-live" variant="solid" tone="primary" size="sm">Send</button>
        </footer>
      </dialog>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Size</label>
        <select size="sm" bind:value={size} label="Size">
          <option value="sm">sm</option>
          <option value="md">md</option>
          <option value="lg">lg</option>
          <option value="full">full</option>
        </select>
      </column>
      <column gap="2">
        <label style(knobLabel)>Backdrop</label>
        <select size="sm" bind:value={backdrop} label="Backdrop">
          <option value="default">default</option>
          <option value="blur">blur</option>
          <option value="solid">solid</option>
        </select>
      </column>
      <column gap="2">
        <label style(knobLabel)>Side</label>
        <select size="sm" bind:value={side} label="Side">
          <option value="center">center</option>
          <option value="start">start</option>
          <option value="end">end</option>
          <option value="top">top</option>
          <option value="bottom">bottom</option>
        </select>
        <p style(mutedXs)>Non-center docks as a sheet.</p>
      </column>
      <column gap="2">
        <label style(knobLabel)>Dismissible</label>
        <row gap="2" align="center">
          <switch bind:checked={canDismiss} size="sm" tone="primary" label="Dismissible"></switch>
          <span style(sizeValue)>{canDismiss() ? 'ESC + backdrop' : 'locked'}</span>
        </row>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="center" style(specimen)>
      <button size="sm" tone="primary" command="show-modal" commandfor="doc-dlg-usage">Open dialog</button>
      <dialog id="doc-dlg-usage" size="md">
        <header>Invite to workspace</header>
        <section>
          <p style(mutedSm)>Header / section / footer anatomy.</p>
        </section>
        <footer>
          <button command="close" commandfor="doc-dlg-usage" variant="ghost" size="sm">Cancel</button>
          <button command="close" commandfor="doc-dlg-usage" variant="solid" tone="primary" size="sm">Send</button>
        </footer>
      </dialog>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Modal invoker"
        lead={'Native command="show-modal" / close — zero JS.'}
        code={dlgExModal()}
      >
        <button size="sm" tone="destructive" command="show-modal" commandfor="doc-dlg-ex-modal">Delete…</button>
        <dialog id="doc-dlg-ex-modal" size="md">
          <header>Delete project</header>
          <section>
            <p style(mutedSm)>This cannot be undone.</p>
          </section>
          <footer>
            <button command="close" commandfor="doc-dlg-ex-modal" variant="ghost" size="sm">Cancel</button>
            <button command="close" commandfor="doc-dlg-ex-modal" tone="destructive" size="sm">Delete</button>
          </footer>
        </dialog>
      </doc-example>
      <doc-example
        title="Sheet"
        lead={'side="end" docks to an edge — same primitive, not a second element.'}
        code={dlgExSheet()}
      >
        <button size="sm" variant="outline" command="show-modal" commandfor="doc-dlg-ex-sheet">Open sheet</button>
        <dialog id="doc-dlg-ex-sheet" side="end" size="sm">
          <header>Filters</header>
          <section>
            <p style(mutedSm)>Sheet content</p>
          </section>
          <footer>
            <button command="close" commandfor="doc-dlg-ex-sheet" variant="ghost" size="sm">Close</button>
          </footer>
        </dialog>
      </doc-example>
      <doc-example
        title="Locked dismiss"
        lead="dismissible=false for destructive confirms."
        code={dlgExLock()}
      >
        <button size="sm" variant="outline" command="show-modal" commandfor="doc-dlg-ex-lock">Open locked</button>
        <dialog id="doc-dlg-ex-lock" dismissible="false">
          <header>Confirm action</header>
          <section>
            <p style(mutedSm)>ESC and backdrop will not close this dialog.</p>
          </section>
          <footer>
            <button command="close" commandfor="doc-dlg-ex-lock" variant="solid" tone="primary" size="sm">Acknowledge</button>
          </footer>
        </dialog>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
