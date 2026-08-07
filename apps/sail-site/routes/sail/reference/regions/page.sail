<!--
  /sail/reference/regions — region/expression capability matrix (CHANNELS-1 WI10).
  Rendered from src/language-surface.gen.ts (emitted by `sail-compiler prop-registry`,
  sourced from sail_compiler::region_capability::REGION_CAPABILITY_MATRIX).
-->
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import {
  REGION_CAPABILITY_MATRIX,
  CONTROLLER_SIZE_THRESHOLD,
} from '../../../../src/language-surface.gen'

const REGIONS = ['scope', 'controller', 'template', 'chain'] as const

const GROUPS = REGIONS.map((region) => ({
  region,
  rows: REGION_CAPABILITY_MATRIX.filter((r) => r.region === region),
}))

export default controller((_ctx: ControllerScope<Scope>) => ({
  groups: () => GROUPS,
  rowCount: () => REGION_CAPABILITY_MATRIX.length,
  sizeThreshold: () => CONTROLLER_SIZE_THRESHOLD,
}))
</controller>

<column gap="6" style({ box: minW(0) })>
  <column gap="5" style({ box: maxW(breakpoint-tablet) })>
    <p token="doc-eyebrow">Reference</p>
    <h1>Region capability matrix</h1>
    <p token="doc-lead">
      What is legal in each region of a <code>.sail</code> file — scope,
      controller, template, and the Style chain grammar — and what enforces it.
      The table is the compiler SSOT (<code>region_capability</code>); every row
      cites a lint rule, a parse gate, or an honest residual gap.
    </p>
    <p style({ paint: fg(muted-foreground), text: fontSize(sm) })>
      Thin-controller policy: a setup body of
      <strong>{sizeThreshold()}+</strong> non-empty, non-comment lines emits
      <code>sail-controller-size</code> (policy warning, default on).
      {rowCount()} matrix cells.
    </p>
  </column>

  <for {groups() as g} key={g.region}>
    <column gap="3">
      <h2>{g.region}</h2>
      <column gap="2">
        <for {g.rows as row} key={row.capability}>
          <card padding="sm" style({ box: minW(0) })>
            <column gap="1">
              <row gap="3" align="center">
                <strong>{row.capability}</strong>
                <code style({ paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })>legal: {row.legal}</code>
                <code style({ paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })>{row.status}</code>
              </row>
              <small style({ box: maxW(breakpoint-tablet), paint: fg(muted-foreground) })>{row.notes}</small>
              <small style({ paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })>{row.enforcement}</small>
            </column>
          </card>
        </for>
      </column>
    </column>
  </for>

  <row>
    <a variant="default" href="/sail/reference/scope">← Scope grammar</a>
    <a variant="solid" href="/sail/reference">Reference index →</a>
  </row>
</column>
