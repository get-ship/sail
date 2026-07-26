<!--
  /sail/reference/scope — every <scope> grammar construct, with a worked
  example that actually compiles (compiled by scope_manifest's own Rust
  tests, never merely asserted here). Rendered from
  src/language-surface.gen.ts (emitted by `sail-compiler prop-registry`).
-->
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { SCOPE_GRAMMAR, SCOPE_REFUSALS } from '../../../../src/language-surface.gen'

export default controller((_ctx: ControllerScope<Scope>) => ({
  grammar: () => SCOPE_GRAMMAR,
  refusals: () => SCOPE_REFUSALS,
}))
</controller>

<column gap="6" sx={style.minW(0)}>
  <column sx={style.maxW(breakpoint-tablet)} gap="5">
    <p role="doc-eyebrow">Reference</p>
    <h1>&lt;scope&gt; grammar</h1>
    <p role="doc-lead">
      Everything a <code>&lt;scope&gt;</code> block can declare: props forms,
      type aliases, and the variants table. Nothing in the block executes —
      it is types and literals only.
    </p>
  </column>

  <column gap="4">
    {#for g of grammar() key={g.form}}
      <column gap="2">
        <row gap="3" align="center">
          <h3>{g.form}</h3>
          <code sx={style.fg(muted-foreground)}>{g.syntax}</code>
        </row>
        <p sx={style.maxW(breakpoint-tablet)}>{g.summary}</p>
        <code-block code={g.example} lang="sail"></code-block>
      </column>
    {/for}
  </column>

  <column gap="3">
    <h3>Refused</h3>
    <p sx={style.maxW(breakpoint-tablet)}>
      <code>&lt;scope&gt;</code> is types and literals only — these member
      kinds are refused with an error naming the alternative.
    </p>
    <column gap="3">
      {#for r of refusals() key={r.syntax}}
        <column gap="1">
          <code sx={style.fg(destructive)}>{r.syntax}</code>
          <small sx={style.fg(muted-foreground)}>→ {r.instead}</small>
        </column>
      {/for}
    </column>
  </column>

  <row role="navigation">
    <a variant="default" href="/sail/reference/template">← Template</a>
    <a variant="solid" href="/sail/reference/runtime">Runtime →</a>
  </row>
</column>
