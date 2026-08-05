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

<column gap="6" sx={style.minW(0)}>
  <column sx={style.maxW(breakpoint-tablet)} gap="5">
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
    <p sx={style.maxW(breakpoint-tablet).fontSize(sm).fg(muted-foreground)}>
      Free <code>aria-*</code> / free ARIA <code>role=</code> are refused
      (<code>sail-no-free-aria</code>). Icon-only buttons require
      <code>label</code>.
    </p>
    <grid cols="1" gap="2">
      <for {semanticProps()} as="p" key={p.name}>
        <card padding="sm" sx={style.minW(0)}>
          <column gap="1">
            <row gap="2" align="center">
              <code>{p.name}</code>
              <small sx={style.fg(muted-foreground)}>→ {p.aria}</small>
              <badge size="sm">{p.kind}</badge>
            </row>
            <small sx={style.fg(muted-foreground).maxW(panel-max-w-sm)}>{p.summary}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>Host anatomy ({anatomyCount()}) — compiler-stamped</h3>
    <p sx={style.maxW(breakpoint-tablet).fontSize(sm).fg(muted-foreground)}>
      Fixed roles / live regions / labels on L1 hosts. Authors never write
      these attributes for listed elements — the compiler stamps them.
    </p>
    <grid cols="2" gap="2">
      <for {anatomyRows()} as="a" key={a.tag}>
        <card padding="sm" sx={style.minW(0)}>
          <column gap="1">
            <code>{a.tag}</code>
            <small sx={style.fg(muted-foreground).maxW(panel-max-w-sm)}>{a.summary}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>Authoring bans ({banCount()})</h3>
    <p sx={style.maxW(breakpoint-tablet).fontSize(sm).fg(muted-foreground)}>
      Constructs the language refuses, with the rule id that fails closed.
    </p>
    <grid cols="1" gap="2">
      <for {bans()} as="b" key={b.rule}>
        <card padding="sm" sx={style.minW(0)}>
          <column gap="1">
            <row gap="2" align="center">
              <code sx={style.fg(destructive)}>{b.syntax}</code>
              <badge size="sm">{b.rule}</badge>
            </row>
            <small sx={style.fg(muted-foreground).maxW(panel-max-w-sm)}>→ {b.instead}</small>
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
