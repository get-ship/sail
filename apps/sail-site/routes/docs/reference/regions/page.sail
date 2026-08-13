<!--
  /docs/reference/regions — region/expression capability matrix (compiler SSOT).
-->
<scope>
  style root = { box: size({ min-w: 0 }) }
  style intro = { box: size({ max-w: breakpoint-tablet }) }
  style eyebrow = { paint: ink({ color: accent }), text: type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style lead = { paint: ink({ color: muted-foreground }), text: type({ size: lg, leading: relaxed }) }
  style meta = { paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
  style cardMin = { box: size({ min-w: 0 }) }
  style monoMuted = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style notes = { box: size({ max-w: breakpoint-tablet }), paint: ink({ color: muted-foreground }) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import {
  REGION_CAPABILITY_MATRIX,
  CONTROLLER_SIZE_THRESHOLD,
} from '../../../../src/compositions/language-surface'

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

<column gap="6" style(root)>
  <column gap="5" style(intro)>
    <p style(eyebrow)>Reference</p>
    <h1>Region capability matrix</h1>
    <p style(lead)>
      What is legal in each region of a <code>.sail</code> file — scope,
      controller, template, and the Style chain grammar — and what enforces it.
      The table is the compiler SSOT (<code>region_capability</code>); every row
      cites a lint rule, a parse gate, or an honest residual gap.
    </p>
    <p style(meta)>
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
          <card padding="sm" style(cardMin)>
            <column gap="1">
              <row gap="3" align="center">
                <strong>{row.capability}</strong>
                <code style(monoMuted)>legal: {row.legal}</code>
                <code style(monoMuted)>{row.status}</code>
              </row>
              <small style(notes)>{row.notes}</small>
              <small style(monoMuted)>{row.enforcement}</small>
            </column>
          </card>
        </for>
      </column>
    </column>
  </for>

  <row>
    <a variant="default" href="/docs/reference/scope">← Scope grammar</a>
    <a variant="solid" href="/docs/reference">Reference index →</a>
  </row>
</column>
