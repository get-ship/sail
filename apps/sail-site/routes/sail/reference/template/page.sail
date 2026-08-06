<!--
  /sail/reference/template — every template directive, every on: modifier,
  every on: event, every bind: target, retired directives, and the block vocabulary. Rendered
  from src/language-surface.gen.ts (emitted by `sail-compiler prop-registry`).
-->
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import {
  DIRECTIVES,
  EVENT_NAMES,
  EVENT_MODIFIERS,
  BIND_TARGETS,
  FOREIGN_IDIOMS,
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
  foreignIdioms: () => FOREIGN_IDIOMS,
  blockRows: () => BLOCK_ROWS,
}))
</controller>

<column gap="6">
  style({ box: minW(0) })

  <column gap="5">
    style({ box: maxW(breakpoint-tablet) })

    <p token="doc-eyebrow">Reference</p>
    <h1>Template directives</h1>
    <p token="doc-lead">
      Every attribute-position directive the parser accepts, in one table.
    </p>
  </column>

  <column gap="3">
    <grid cols="1" gap="2">
      <for {directives() as d} key={d.syntax}>
        <card padding="sm">
          style({ box: minW(0) })

          <row gap="3" align="center">
            <code>
              style({ box: minW(40) })
{d.syntax}</code>
            <small>
              style({ paint: fg(muted-foreground) })
{d.summary}</small>
          </row>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>on: events</h3>
    <p>
      style({ box: maxW(breakpoint-tablet) })

      Standard DOM event names only. Component moments are typed
      <code>on-*</code> callback props (e.g. <code>on-select=&#123;fn&#125;</code>);
      form mutation moments are <code>on-success</code> /
      <code>on-failure</code>. Props flow down; callbacks report up;
      <code>bind:</code> syncs a value — there is no custom-event bus.
    </p>
    <grid cols="2" gap="2">
      <for {eventNames() as e} key={e.name}>
        <card padding="sm">
          style({ box: minW(0) })

          <column gap="1">
            <code>on:{e.name}</code>
            <small>
              style({ box: maxW(panel-max-w-sm), paint: fg(muted-foreground) })
{e.summary}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>on: modifiers</h3>
    <p>
      style({ box: maxW(breakpoint-tablet) })
Chain any number with |, e.g. on:keydown|prevent|key:Enter.</p>
    <grid cols="2" gap="2">
      <for {modifierRows() as m} key={m.key}>
        <card padding="sm">
          style({ box: minW(0) })

          <column gap="1">
            <code>{m.written}{m.arg}</code>
            <small>
              style({ box: maxW(panel-max-w-sm), paint: fg(muted-foreground) })
{m.summary}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>bind: targets</h3>
    <grid cols="3" gap="2">
      <for {bindTargets() as b} key={b.target}>
        <card padding="sm">
          style({ box: minW(0) })

          <column gap="1">
            <code>bind:{b.target}</code>
            <small>
              style({ box: maxW(panel-max-w-sm), paint: fg(muted-foreground) })
{b.summary}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>Blocks</h3>
    <grid cols="2" gap="2">
      <for {blockRows() as b} key={b.open}>
        <card padding="sm">
          style({ box: minW(0) })

          <column gap="1">
            <row gap="2" align="center">
              <code>{b.open}</code>
              <small>
                style({ paint: fg(muted-foreground) })
{b.clauses}</small>
            </row>
            <small>
              style({ box: maxW(panel-max-w-sm), paint: fg(muted-foreground) })
{b.summary}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <column gap="3">
    <h3>Foreign idioms</h3>
    <p>
      style({ box: maxW(breakpoint-tablet) })

      HTML/CSS/JS habits the parser teaches against, with the sail way.
    </p>
    <grid cols="1" gap="2">
      <for {foreignIdioms() as r} key={r.syntax}>
        <card padding="sm">
          style({ box: minW(0) })

          <column gap="1">
            <code>
              style({ paint: fg(destructive) })
{r.syntax}</code>
            <small>
              style({ box: maxW(panel-max-w-sm), paint: fg(muted-foreground) })
→ {r.instead}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <row>
    <a variant="default" href="/sail/reference/actions">← Actions</a>
    <a variant="solid" href="/sail/reference/scope">Scope grammar →</a>
  </row>
</column>
