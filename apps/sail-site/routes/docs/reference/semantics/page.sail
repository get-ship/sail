<!--
  /docs/reference/semantics — semantic a11y props, compiler host anatomy,
  authoring bans. From language-surface.gen (sail-compiler prop-registry).
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import {
  SEMANTIC_PROPS,
  HOST_ANATOMY,
  AUTHORING_BANS,
} from '../../../../src/compositions/language-surface'

function anatomySummary(a: {
  role: string | null
  live: string | null
  atomic: string | null
  label: string | null
}): string {
  const parts: string[] = []
  if (a.role) parts.push(`role=${a.role}`)
  if (a.live) parts.push(`aria-live=${a.live}`)
  if (a.atomic) parts.push(`aria-atomic=${a.atomic}`)
  if (a.label) parts.push(`aria-label=${a.label}`)
  return parts.length > 0 ? parts.join(' · ') : 'structure-only / native tag carries semantics'
}

const ANATOMY_ROWS = HOST_ANATOMY.map((a) => ({
  tag: a.tag,
  summary: anatomySummary(a),
}))

export default controller((_ctx: ControllerScope<Scope>) => ({
  semanticProps: () => SEMANTIC_PROPS,
  semanticCount: () => SEMANTIC_PROPS.length,
  anatomyRows: () => ANATOMY_ROWS,
  anatomyCount: () => HOST_ANATOMY.length,
  bans: () => AUTHORING_BANS,
  banCount: () => AUTHORING_BANS.length,
}))
</controller>

<column gap="6">
  <column gap="5">
    <p>Reference</p>
    <h1>Semantics — a11y IR &amp; floor bans</h1>
    <p>
      Authors type semantic props; the compiler stamps the correct ARIA.
      Host anatomy is compiler behavior, not free author API. Floor bans
      refuse escape hatches by rule id.
    </p>
  </column>

  <column gap="3">
    <h3>Semantic props ({semanticCount()})</h3>
    <p>
      Free <code>aria-*</code> / free ARIA <code>role=</code> are refused
      (<code>sail-no-free-aria</code>). Icon-only buttons require
      <code>label</code>.
    </p>
    <grid cols="1" gap="2">
      <for {semanticProps() as p} key={p.name}>
        <card padding="sm">
          <column gap="1">
            <row gap="2" align="center">
              <code>{p.name}</code>
              <small>→ {p.aria}</small>
              <badge size="sm">{p.kind}</badge>
            </row>
            <small>{p.summary}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>Host anatomy ({anatomyCount()}) — compiler-stamped</h3>
    <p>
      Fixed roles / live regions / labels on L1 hosts. Authors never write
      these attributes for listed elements — the compiler stamps them.
      The compiler also stamps <code>data-sail-host</code> as closed-widget
      part identity (not an author API; refused by
      <code>sail-no-data-attr</code>). App tests use role / name / text.
    </p>
    <grid cols="2" gap="2">
      <for {anatomyRows() as a} key={a.tag}>
        <card padding="sm">
          <column gap="1">
            <code>{a.tag}</code>
            <small>{a.summary}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>Selector contract (app tests)</h3>
    <p>
      Versioned public API for locating UI in behavioral tests
      (<code>@ship/sail-test</code>). Same tables as above — regenerated
      from compiler SSOT (<code>SEMANTIC_PROPS</code> +
      <code>HOST_ANATOMY</code>), not hand lists.
    </p>
    <card padding="sm">
      <column gap="2">
        <p>
          <strong>Role</strong> — native tags (<code>button</code>,
          <code>nav</code>, <code>h1</code>…) or host anatomy stamps
          (table above). Never free <code>role=</code> in app markup.
        </p>
        <p>
          <strong>Name</strong> — <code>label</code> prop →
          <code>aria-label</code>, labelled text, or visible text.
        </p>
        <p>
          <strong>State</strong> — semantic props
          <code>expanded</code> / <code>selected</code> /
          <code>pressed</code> / <code>current</code> / <code>invalid</code>
          / <code>hidden</code> / <code>live</code>.
        </p>
        <p>
          Prefer <code>getByRole(root, role, &#123; name, …state &#125;)</code>.
          Do not query <code>data-sail-host</code> or other
          <code>data-*</code> in app suites. Force query lifecycle with
          <code>mockQueryState</code>; force <code>&lt;when&gt;</code> via
          viewport helpers — no private runtime poke.
        </p>
      </column>
    </card>
  </column>

  <column gap="3">
    <h3>Authoring bans ({banCount()})</h3>
    <p>
      Constructs the language refuses, with the rule id that fails closed.
    </p>
    <grid cols="1" gap="2">
      <for {bans() as b} key={b.rule}>
        <card padding="sm">
          <column gap="1">
            <row gap="2" align="center">
              <code>{b.syntax}</code>
              <badge size="sm">{b.rule}</badge>
            </row>
            <small>→ {b.instead}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <row>
    <a variant="default" href="/docs/reference">← Reference index</a>
    <a variant="solid" href="/docs/reference/elements">Elements →</a>
  </row>
</column>
