<!--
  <doc-pagination> — first-class docs host for /components/pagination.
  Specimen left, knobs right — value string; onPageChange is live on stage.
-->

<controller>
import type { Scope } from './doc-pagination.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const pageNo = signal('2')
  const goPage = (detail: { page?: string; value?: string }): void =>
    pageNo.set(detail?.page ?? detail?.value ?? '1')
  const pgMeta = derived(() => {
    return `value "${pageNo()}" · aria-current on matching <page> · onPageChange → your list`
  })
  const entry = DETAILS.pagination
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const pgExBasic = () =>
    `// page = signal('2')\n// go = (e) => page.set(e.page)\n<pagination value={page()} on-page-change={go}>\n  <page value="1">1</page>\n  <page value="2">2</page>\n  <page value="3">3</page>\n</pagination>`
  const pgExPrevNext = () =>
    `<pagination value="3" on-page-change={go}>\n  <page value="2">‹</page>\n  <page value="1">1</page>\n  <page value="2">2</page>\n  <page value="3">3</page>\n  <page value="4">4</page>\n  <page value="4">›</page>\n</pagination>`
  const pgExNote = () =>
    `<!-- page cells are not a hub tile; size Style on each <page> -->\n<pagination value="1">\n  <page value="1" size="sm">1</page>\n</pagination>`
  return {
    pageNo,
    goPage,
    pgMeta,
    heading,
    blurb,
    props,
    usageCode,
    pgExBasic,
    pgExPrevNext,
    pgExNote,
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
            <pagination value={pageNo()} on-page-change={goPage}>
              <page value="1">1</page>
              <page value="2">2</page>
              <page value="3">3</page>
              <page value="4">4</page>
              <page value="5">5</page>
            </pagination>
            <p style(monoXs)>{pgMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Value</label>
            <select size="sm" bind:value={pageNo} label="Value">
              <option value="1">1</option>
              <option value="2">2</option>
              <option value="3">3</option>
              <option value="4">4</option>
              <option value="5">5</option>
            </select>
            <p style(mutedXs)>
              Current page string. Clicking a <code>page</code> fires
              <code>onPageChange</code> with <code>{'{ page }'}</code>.
            </p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <p style(mutedXs)>
              None — no <code>component.pagination.*</code> leaves. Page controls reuse free chrome.
            </p>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <column gap="2">
        <h3 style(exampleH)>Controlled pages</h3>
        <p style(exampleLead)>
          Author the page list; the host keeps <code>aria-current</code> in sync with
          <code>value</code>.
        </p>
        <pagination value={pageNo()} on-page-change={goPage}>
          <page value="1">1</page>
          <page value="2">2</page>
          <page value="3">3</page>
        </pagination>
        <code-block code={pgExBasic()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Prev / next as pages</h3>
        <p style(exampleLead)>
          Prev and next are just <code>page</code> cells whose value is the target page.
        </p>
        <pagination value="3" on-page-change={goPage}>
          <page value="2">‹</page>
          <page value="1">1</page>
          <page value="2">2</page>
          <page value="3">3</page>
          <page value="4">4</page>
          <page value="4">›</page>
        </pagination>
        <code-block code={pgExPrevNext()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Size on each page</h3>
        <p style(exampleLead)>
          <code>page</code> is not a hub tile. Size Style is authored per cell.
        </p>
        <code-block code={pgExNote()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
