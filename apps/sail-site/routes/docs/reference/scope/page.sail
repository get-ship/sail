<!--
  /docs/reference/scope — every &lt;scope&gt; grammar construct with a compiling example.
-->
<scope>
  style root = { box: size({ min-w: 0 }) }
  style intro = { box: size({ max-w: breakpoint-tablet }) }
  style eyebrow = { paint: ink({ color: accent }), text: type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style lead = { paint: ink({ color: muted-foreground }), text: type({ size: lg, leading: relaxed }) }
  style monoMuted = { paint: ink({ color: muted-foreground }) }
  style formSummary = { box: size({ max-w: breakpoint-tablet }) }
  style refuseSyntax = { paint: ink({ color: destructive }) }
  style refuseAlt = { paint: ink({ color: muted-foreground }) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { SCOPE_GRAMMAR, SCOPE_REFUSALS } from '../../../../src/compositions/language-surface'

export default controller((_ctx: ControllerScope<Scope>) => ({
  grammar: () => SCOPE_GRAMMAR,
  refusals: () => SCOPE_REFUSALS,
}))
</controller>

<column gap="6" style(root)>
  <column gap="5" style(intro)>
    <p style(eyebrow)>Reference</p>
    <h1>&lt;scope&gt; grammar</h1>
    <p style(lead)>
      Everything a <code>&lt;scope&gt;</code> block can declare: props forms,
      type aliases, and the variants table. Nothing in the block executes —
      it is types and literals only.
    </p>
  </column>

  <column gap="4">
    <for {grammar() as g} key={g.form}>
      <column gap="2">
        <row gap="3" align="center">
          <h3>{g.form}</h3>
          <code style(monoMuted)>{g.syntax}</code>
        </row>
        <p style(formSummary)>{g.summary}</p>
        <code-block code={g.example} lang="sail"></code-block>
      </column>
    </for>
  </column>

  <column gap="3">
    <h3>Refused</h3>
    <p style(formSummary)>
      <code>&lt;scope&gt;</code> is types and literals only — these member
      kinds are refused with an error naming the alternative.
    </p>
    <column gap="3">
      <for {refusals() as r} key={r.syntax}>
        <column gap="1">
          <code style(refuseSyntax)>{r.syntax}</code>
          <small style(refuseAlt)>→ {r.instead}</small>
        </column>
      </for>
    </column>
  </column>

  <row>
    <a variant="default" href="/docs/reference/template">← Template</a>
    <a variant="solid" href="/docs/reference/runtime">Runtime →</a>
  </row>
</column>
