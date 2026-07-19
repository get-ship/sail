<!--
  /sail/reference/sx — the complete sx vocabulary: atoms, refusals, state/
  condition prefixes, and the group-/state-<value>:/(--var) extras. Rendered
  from src/language-surface.gen.ts (emitted by `sail-compiler prop-registry`).
-->
<controller>
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import {
  SX_ATOMS,
  SX_STATE_PREFIXES,
  SX_CONDITION_PREFIXES,
  SX_REFUSALS,
  SX_EXTRA_FORMS,
} from '../../../../src/language-surface.gen'

// A few atoms register under the same `prop` name in more than one table
// (e.g. a spacing sense and a color sense both named `border`) — merge
// their value lists under one card instead of letting the later
// registration silently shadow the earlier one in a keyed {#for}.
const SX_BY_PROP = new Map<string, string[]>()
for (const a of SX_ATOMS) {
  const existing = SX_BY_PROP.get(a.prop)
  if (existing) {
    existing.push(...a.values)
  } else {
    SX_BY_PROP.set(a.prop, [...a.values])
  }
}
const SX_ROWS = [...SX_BY_PROP.entries()].map(([prop, values]) => ({
  prop,
  values: values.join(' · '),
}))

export default controller((_ctx: ControllerScope<Scope>) => ({
  sxRows: () => SX_ROWS,
  sxCount: () => SX_ATOMS.length,
  statePrefixes: () => SX_STATE_PREFIXES,
  conditionPrefixes: () => SX_CONDITION_PREFIXES,
  refusals: () => SX_REFUSALS,
  refusalCount: () => SX_REFUSALS.length,
  extraForms: () => SX_EXTRA_FORMS,
}))
</controller>

<column gap="6" sx="min-w-0">
  <column sx="max-w-breakpoint-tablet" gap="5">
    <p role="doc-eyebrow">Reference</p>
    <h1>sx — {sxCount()} atoms</h1>
    <p role="doc-lead">
      <code>sx="name-value name-value …"</code>, static only. Every value is
      a token from a closed list — the rows below are the complete surface,
      and an off-list value is a compile error that names this same list.
    </p>
  </column>

  <column gap="3">
    <grid cols="2" gap="2">
      {#for a of sxRows() key={a.prop}}
        <card padding="sm" sx="min-w-0">
          <column gap="1">
            <code>{a.prop}:</code>
            <small sx="text-muted-foreground max-w-panel-max-w-sm">{a.values}</small>
          </column>
        </card>
      {/for}
    </grid>
  </column>

  <column gap="3">
    <h3>Not sx — use this ({refusalCount()})</h3>
    <p sx="max-w-breakpoint-tablet">
      The refused table is the OTHER half of the vocabulary. Every entry
      below is an atom a Tailwind prior will reach for and sx refuses by
      name; each names the element or attribute to use instead. The mission
      is never "find another way" — it is "use this."
    </p>
    <grid cols="2" gap="2">
      {#for r of refusals() key={r.atom}}
        <card padding="sm" sx="min-w-0">
          <column gap="1">
            <code sx="text-destructive">sx={r.atom}</code>
            <small sx="text-muted-foreground max-w-panel-max-w-sm">→ {r.instead}</small>
          </column>
        </card>
      {/for}
    </grid>
  </column>

  <column gap="3">
    <h3>State prefixes</h3>
    <grid cols="3" gap="2">
      {#for s of statePrefixes() key={s.prefix}}
        <row gap="3" sx="px-3 py-1 rounded-md" align="center">
          <code sx="min-w-9">{s.prefix}:</code>
          <small sx="text-muted-foreground font-mono max-w-ch-24">{s.selector}</small>
        </row>
      {/for}
    </grid>
    <p sx="max-w-breakpoint-tablet">
      Ancestor propagation: mark the ancestor with bare <code>group</code>,
      then <code>group-hover:</code> (any state prefix) on the descendant.
    </p>
  </column>

  <column gap="3">
    <h3>Condition prefixes</h3>
    <grid cols="3" gap="2">
      {#for c of conditionPrefixes() key={c.prefix}}
        <row gap="3" sx="px-3 py-1 rounded-md" align="center">
          <code sx="min-w-9">{c.prefix}:</code>
          <small sx="text-muted-foreground font-mono max-w-ch-24">{c.media}</small>
        </row>
      {/for}
    </grid>
  </column>

  <column gap="3">
    <h3>Extra forms</h3>
    <p sx="max-w-breakpoint-tablet">
      Grammar rather than a closed name table: the ancestor marker, the
      component-state prefix, and the custom-property value read.
    </p>
    <grid cols="2" gap="2">
      {#for f of extraForms() key={f.form}}
        <card padding="sm" sx="min-w-0">
          <column gap="1">
            <code>{f.form}</code>
            <small sx="text-muted-foreground max-w-panel-max-w-sm">{f.doc}</small>
          </column>
        </card>
      {/for}
    </grid>
  </column>

  <row role="navigation">
    <a variant="default" href="/sail/reference/layout">← Layout keywords</a>
    <a variant="solid" href="/sail/reference/motion">Motion →</a>
  </row>
</column>
