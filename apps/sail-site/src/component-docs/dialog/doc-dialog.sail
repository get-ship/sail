<!--
  <doc-dialog> — first-class docs host for /components/dialog.
  Specimen: static open anatomy + invoker demo. size · backdrop · side · dismissible.
-->

<controller>
import type { Scope } from './doc-dialog.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const size = signal('md')
  const backdrop = signal('default')
  const side = signal('center')
  const dismissible = signal(true)
  const dlgMeta = derived(() => {
    return `${size()} · backdrop ${backdrop()} · side ${side()} · dismissible ${dismissible()} · component.dialog.*`
  })
  const entry = DETAILS.dialog
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const dlgExModal = () =>
    `<button command="show-modal" commandfor="confirm">Delete…</button>\n<dialog id="confirm" size="md">\n  <header>Delete project</header>\n  <section>This cannot be undone.</section>\n  <footer>\n    <button command="close" commandfor="confirm" variant="ghost">Cancel</button>\n    <button command="close" commandfor="confirm" tone="destructive">Delete</button>\n  </footer>\n</dialog>`
  const dlgExSheet = () =>
    `<dialog side="end" size="sm">\n  <header>Filters</header>\n  <section>…</section>\n  <footer>…</footer>\n</dialog>`
  const dlgExLock = () =>
    `<dialog dismissible={false}>\n  <!-- ESC + backdrop cannot close; user must pick an action -->\n</dialog>`
  return {
    size, backdrop, side, dismissible, dlgMeta,
    heading, blurb, props, usageCode, dlgExModal, dlgExSheet, dlgExLock,
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
  style fullW = { box: w(full) }
  style swatchRow = { box: w(full) }
  style swatch = {
    box: size(6).shrink(0),
    paint: radius(full).borderWidth(1).border(border).cursor(pointer),
  }
  style swatchRest = { paint: bg(surface-1) }
  style swatchPrimary = { paint: bg(primary) }
  style swatchInfo = { paint: bg(info) }
  style swatchSuccess = { paint: bg(success) }
  style swatchWarning = { paint: bg(warning) }
  style swatchDestructive = { paint: bg(destructive) }
  style swatchAccent = { paint: bg(accent) }
  style swatchMuted = { paint: bg(muted) }
  style centerBox = { box: pad(4), paint: borderWidth(1).border(border-subtle).radius(md) }

  style dlgDemo = { box: w(full).maxW(panel-max-w-md) }
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
          <column gap="3" align="center" style(stageCol)>
            <button size="sm" tone="primary" command="show-modal" commandfor="doc-dlg-live">Open dialog</button>
            <dialog
              id="doc-dlg-live"
              size={size()}
              backdrop={backdrop()}
              side={side()}
              dismissible={dismissible() ? 'true' : 'false'}
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
            <p style(monoXs)>{dlgMeta()}</p>
            <p style(mutedXs)>Static anatomy chrome also in examples — real top-layer open via invoker above.</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
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
              <switch bind:checked={dismissible} size="sm" tone="primary" label="Dismissible"></switch>
              <span style(sizeValue)>{dismissible() ? 'ESC + backdrop' : 'locked'}</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <column gap="1" style(faceRow)>
              <p style(mutedXs)><code>component.dialog.bg / fg / border</code></p>
              <p style(mutedXs)><code>divider · footer · muted-fg</code></p>
            </column>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Modal invoker</h3>
        <p style(exampleLead)>Native <code>command="show-modal"</code> / <code>close</code> — zero JS.</p>
        <code-block code={dlgExModal()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Sheet</h3>
        <p style(exampleLead)><code>side="end"</code> docks to an edge — same primitive, not a second element.</p>
        <code-block code={dlgExSheet()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Locked dismiss</h3>
        <p style(exampleLead)><code>dismissible=false</code> for destructive confirms.</p>
        <code-block code={dlgExLock()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
