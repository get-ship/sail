<!--
  /docs/reference/scope — every &lt;scope&gt; grammar construct with a compiling example.
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { SCOPE_GRAMMAR, SCOPE_REFUSALS } from '../../../../src/compositions/language-surface'

export default controller((_ctx: ControllerScope<Scope>) => ({
  grammar: () => SCOPE_GRAMMAR,
  refusals: () => SCOPE_REFUSALS,
}))
</controller>

<column gap="6">
  <column gap="5">
    <p>Reference</p>
    <h1>&lt;scope&gt; grammar</h1>
    <p>
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
          <code>{g.syntax}</code>
        </row>
        <p>{g.summary}</p>
        <code-block code={g.example} lang="sail"></code-block>
      </column>
    </for>
  </column>

  <column gap="3">
    <h3>Refused</h3>
    <p>
      <code>&lt;scope&gt;</code> is types and literals only — these member
      kinds are refused with an error naming the alternative.
    </p>
    <column gap="3">
      <for {refusals() as r} key={r.syntax}>
        <column gap="1">
          <code>{r.syntax}</code>
          <small>→ {r.instead}</small>
        </column>
      </for>
    </column>
  </column>

  <row>
    <a variant="default" href="/docs/reference/template">← Template</a>
    <a variant="solid" href="/docs/reference/runtime">Runtime →</a>
  </row>
</column>
