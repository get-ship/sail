<!--
  <doc-pagination> — /components/pagination
  Shared chrome: split + snippet copy + usage Code|Live|Split.
  Specimen: value string; onPageChange live.
-->

<controller>
import type { Scope } from './doc-pagination.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const pageNo = signal('2')
  const goPage = (detail: { page?: string; value?: string }): void =>
    pageNo.set(detail?.page ?? detail?.value ?? '1')

  const entry = DETAILS.pagination
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    return `// page = signal('${pageNo()}')\n// go = (e) => page.set(e.page)\n<pagination value={page()} on-page-change={go}>\n  <page value="1">1</page>\n  <page value="2">2</page>\n  <page value="3">3</page>\n  <page value="4">4</page>\n  <page value="5">5</page>\n</pagination>`
  })

  const pgExBasic = () =>
    `// page = signal('2')\n// go = (e) => page.set(e.page)\n<pagination value={page()} on-page-change={go}>\n  <page value="1">1</page>\n  <page value="2">2</page>\n  <page value="3">3</page>\n</pagination>`
  const pgExPrevNext = () =>
    `<pagination value="3" on-page-change={go}>\n  <page value="2">‹</page>\n  <page value="1">1</page>\n  <page value="2">2</page>\n  <page value="3">3</page>\n  <page value="4">4</page>\n  <page value="4">›</page>\n</pagination>`
  const pgExNote = () =>
    `<!-- page cells are not a hub tile; size Style on each <page> -->\n<pagination value="1">\n  <page value="1" size="sm">1</page>\n</pagination>`

  return {
    pageNo,
    goPage,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    pgExBasic,
    pgExPrevNext,
    pgExNote,
  }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style specimen = { box: w(full).maxW(panel-max-w-md).pad(2).overflow(visible) }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style exampleBlock = { box: w(full).padT(2).padB(4).overflow(visible) }
  style sectionLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).textCase(uppercase).tracking(wide),
  }
</scope>

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
      <pagination value={pageNo()} on-page-change={goPage}>
        <page value="1">1</page>
        <page value="2">2</page>
        <page value="3">3</page>
        <page value="4">4</page>
        <page value="5">5</page>
      </pagination>
    </column>

    <column slot="knobs" gap="4" align="stretch">
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
    </column>

    <column slot="usage-preview" gap="3" align="center" style(specimen)>
      <pagination value={pageNo()} on-page-change={goPage}>
        <page value="1">1</page>
        <page value="2">2</page>
        <page value="3">3</page>
      </pagination>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Controlled pages"
        lead="Author the page list; the host keeps aria-current in sync with value."
        code={pgExBasic()}
      >
        <pagination value={pageNo()} on-page-change={goPage}>
          <page value="1">1</page>
          <page value="2">2</page>
          <page value="3">3</page>
        </pagination>
      </doc-example>
      <doc-example
        title="Prev / next as pages"
        lead="Prev and next are just page cells whose value is the target page."
        code={pgExPrevNext()}
      >
        <pagination value="3" on-page-change={goPage}>
          <page value="2">‹</page>
          <page value="1">1</page>
          <page value="2">2</page>
          <page value="3">3</page>
          <page value="4">4</page>
          <page value="4">›</page>
        </pagination>
      </doc-example>
      <doc-example
        title="Size on each page"
        lead="page is not a hub tile. Size Style is authored per cell."
        code={pgExNote()}
      >
        <pagination value="1">
          <page value="1" size="sm">1</page>
          <page value="2" size="sm">2</page>
          <page value="3" size="sm">3</page>
        </pagination>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
