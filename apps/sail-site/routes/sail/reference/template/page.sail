<!--
  /sail/reference/template — every template directive, every on: modifier,
  every on: event, every bind: target, retired directives, and the block vocabulary. Rendered
  from src/language-surface.gen.ts (emitted by `sail-compiler prop-registry`).
-->
<controller>
import { controller, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import {
  DIRECTIVES,
  EVENT_NAMES,
  EVENT_MODIFIERS,
  BIND_TARGETS,
  RETIRED_DIRECTIVES,
  BLOCKS,
} from '../../../../src/language-surface.gen'

const MODIFIER_ROWS = EVENT_MODIFIERS.map((m) => ({
  key: m.canonical,
  written: m.alias ? `|${m.canonical} (|${m.alias})` : `|${m.canonical}`,
  arg: m.arg ? `:${m.arg}` : '',
  summary: m.summary,
}))

const BLOCK_ROWS = BLOCKS.map((b) => ({
  open: b.open,
  clauses: b.clauses.length > 0 ? b.clauses.join(' · ') : '—',
  summary: b.summary,
}))

export default controller((_ctx: ControllerScope<Scope>) => ({
  directives: () => DIRECTIVES,
  eventNames: () => EVENT_NAMES,
  modifierRows: () => MODIFIER_ROWS,
  bindTargets: () => BIND_TARGETS,
  retired: () => RETIRED_DIRECTIVES,
  blockRows: () => BLOCK_ROWS,
}))
</controller>

<column gap="6" sx="min-w-0">
  <column sx="max-w-breakpoint-tablet" gap="5">
    <p role="doc-eyebrow">Reference</p>
    <h1>Template directives</h1>
    <p role="doc-lead">
      Every attribute-position directive the parser accepts, in one table.
    </p>
  </column>

  <column gap="3">
    <grid cols="1" gap="2">
      {#for d of directives() key={d.syntax}}
        <card padding="sm" sx="min-w-0">
          <row gap="3" align="center">
            <code sx="min-w-40">{d.syntax}</code>
            <small sx="text-muted-foreground">{d.summary}</small>
          </row>
        </card>
      {/for}
    </grid>
  </column>

  <column gap="3">
    <h3>on: events</h3>
    <p sx="max-w-breakpoint-tablet">
      Web DOM event names plus framework custom events the runtime fires.
      <code>sail-*</code> / <code>ship-*</code> prefixes and namespaced
      <code>tag:event</code> names stay open without a row here.
    </p>
    <grid cols="2" gap="2">
      {#for e of eventNames() key={e.name}}
        <card padding="sm" sx="min-w-0">
          <column gap="1">
            <code>on:{e.name}</code>
            <small sx="text-muted-foreground max-w-panel-max-w-sm">{e.summary}</small>
          </column>
        </card>
      {/for}
    </grid>
  </column>

  <column gap="3">
    <h3>on: modifiers</h3>
    <p sx="max-w-breakpoint-tablet">Chain any number with |, e.g. on:keydown|prevent|key:Enter.</p>
    <grid cols="2" gap="2">
      {#for m of modifierRows() key={m.key}}
        <card padding="sm" sx="min-w-0">
          <column gap="1">
            <code>{m.written}{m.arg}</code>
            <small sx="text-muted-foreground max-w-panel-max-w-sm">{m.summary}</small>
          </column>
        </card>
      {/for}
    </grid>
  </column>

  <column gap="3">
    <h3>bind: targets</h3>
    <grid cols="3" gap="2">
      {#for b of bindTargets() key={b.target}}
        <card padding="sm" sx="min-w-0">
          <column gap="1">
            <code>bind:{b.target}</code>
            <small sx="text-muted-foreground max-w-panel-max-w-sm">{b.summary}</small>
          </column>
        </card>
      {/for}
    </grid>
  </column>

  <column gap="3">
    <h3>Blocks</h3>
    <grid cols="2" gap="2">
      {#for b of blockRows() key={b.open}}
        <card padding="sm" sx="min-w-0">
          <column gap="1">
            <row gap="2" align="center">
              <code>{b.open}</code>
              <small sx="text-muted-foreground">{b.clauses}</small>
            </row>
            <small sx="text-muted-foreground max-w-panel-max-w-sm">{b.summary}</small>
          </column>
        </card>
      {/for}
    </grid>
  </column>

  <column gap="3">
    <h3>Retired</h3>
    <p sx="max-w-breakpoint-tablet">Directives the parser refuses by name, with what to use instead.</p>
    <grid cols="1" gap="2">
      {#for r of retired() key={r.syntax}}
        <card padding="sm" sx="min-w-0">
          <column gap="1">
            <code sx="text-destructive">{r.syntax}</code>
            <small sx="text-muted-foreground max-w-panel-max-w-sm">→ {r.instead}</small>
          </column>
        </card>
      {/for}
    </grid>
  </column>

  <row role="navigation">
    <a variant="default" href="/sail/reference/actions">← Actions</a>
    <a variant="solid" href="/sail/reference/scope">Scope grammar →</a>
  </row>
</column>
