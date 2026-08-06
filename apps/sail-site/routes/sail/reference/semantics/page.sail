<!--
  /sail/reference/semantics — semantic a11y props, compiler host anatomy,
  and authoring bans. Rendered from src/language-surface.gen.ts (emitted by
  `sail-compiler prop-registry`). Nothing here is hand-listed.
-->
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import {
  SEMANTIC_PROPS,
  HOST_ANATOMY,
  AUTHORING_BANS,
} from '../../../../src/language-surface.gen'

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
  style({ box: minW(0) })

  <column gap="5">
    style({ box: maxW(breakpoint-tablet) })

    <p token="doc-eyebrow">Reference</p>
    <h1>Semantics — a11y IR &amp; floor bans</h1>
    <p token="doc-lead">
      Authors type semantic props; the compiler stamps the correct ARIA.
      Host anatomy is compiler behavior, not free author API. Floor bans
      refuse escape hatches by rule id.
    </p>
  </column>

  <column gap="3">
    <h3>Semantic props ({semanticCount()})</h3>
    <p>
      style({ box: maxW(breakpoint-tablet), paint: fg(muted-foreground), text: fontSize(sm) })

      Free <code>aria-*</code> / free ARIA <code>role=</code> are refused
      (<code>sail-no-free-aria</code>). Icon-only buttons require
      <code>label</code>.
    </p>
    <grid cols="1" gap="2">
      <for {semanticProps() as p} key={p.name}>
        <card padding="sm">
          style({ box: minW(0) })

          <column gap="1">
            <row gap="2" align="center">
              <code>{p.name}</code>
              <small>
                style({ paint: fg(muted-foreground) })
→ {p.aria}</small>
              <badge size="sm">{p.kind}</badge>
            </row>
            <small>
              style({ box: maxW(panel-max-w-sm), paint: fg(muted-foreground) })
{p.summary}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>Host anatomy ({anatomyCount()}) — compiler-stamped</h3>
    <p>
      style({ box: maxW(breakpoint-tablet), paint: fg(muted-foreground), text: fontSize(sm) })

      Fixed roles / live regions / labels on L1 hosts. Authors never write
      these attributes for listed elements — the compiler stamps them.
      The compiler also stamps <code>data-sail-host</code> as closed-widget
      part identity (not an author API; refused by
      <code>sail-no-data-attr</code>). App tests use role / name / text.
    </p>
    <grid cols="2" gap="2">
      <for {anatomyRows() as a} key={a.tag}>
        <card padding="sm">
          style({ box: minW(0) })

          <column gap="1">
            <code>{a.tag}</code>
            <small>
              style({ box: maxW(panel-max-w-sm), paint: fg(muted-foreground) })
{a.summary}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>Selector contract (app tests)</h3>
    <p>
      style({ box: maxW(breakpoint-tablet), paint: fg(muted-foreground), text: fontSize(sm) })

      Versioned public API for locating UI in behavioral tests
      (<code>@ship/sail-test</code>). Same tables as above — regenerated
      from compiler SSOT (<code>SEMANTIC_PROPS</code> +
      <code>HOST_ANATOMY</code>), not hand lists.
    </p>
    <card padding="sm">
      style({ box: minW(0) })

      <column gap="2">
        <p>
          style({ text: fontSize(sm) })

          <strong>Role</strong> — native tags (<code>button</code>,
          <code>nav</code>, <code>h1</code>…) or host anatomy stamps
          (table above). Never free <code>role=</code> in app markup.
        </p>
        <p>
          style({ text: fontSize(sm) })

          <strong>Name</strong> — <code>label</code> prop →
          <code>aria-label</code>, labelled text, or visible text.
        </p>
        <p>
          style({ text: fontSize(sm) })

          <strong>State</strong> — semantic props
          <code>expanded</code> / <code>selected</code> /
          <code>pressed</code> / <code>current</code> / <code>invalid</code>
          / <code>hidden</code> / <code>live</code>.
        </p>
        <p>
          style({ paint: fg(muted-foreground), text: fontSize(sm) })

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
      style({ box: maxW(breakpoint-tablet), paint: fg(muted-foreground), text: fontSize(sm) })

      Constructs the language refuses, with the rule id that fails closed.
    </p>
    <grid cols="1" gap="2">
      <for {bans() as b} key={b.rule}>
        <card padding="sm">
          style({ box: minW(0) })

          <column gap="1">
            <row gap="2" align="center">
              <code>
                style({ paint: fg(destructive) })
{b.syntax}</code>
              <badge size="sm">{b.rule}</badge>
            </row>
            <small>
              style({ box: maxW(panel-max-w-sm), paint: fg(muted-foreground) })
→ {b.instead}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <row>
    <a variant="default" href="/sail/reference">← Reference index</a>
    <a variant="solid" href="/sail/reference/elements">Elements →</a>
  </row>
</column>
