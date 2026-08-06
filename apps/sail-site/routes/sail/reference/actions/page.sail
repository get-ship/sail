<!--
  /sail/reference/actions — the use: registry, rendered from
  src/language-surface.gen.ts (emitted by `sail-compiler prop-registry`).
-->
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { PUBLIC_ACTIONS } from '../../../../src/language-surface.gen'

export default controller((_ctx: ControllerScope<Scope>) => ({
  actions: () => PUBLIC_ACTIONS,
  actionCount: () => PUBLIC_ACTIONS.length,
}))
</controller>

<column gap="6">
  style.minW(0)

  <column gap="5">
    style.maxW(breakpoint-tablet)

    <p token="doc-eyebrow">Reference</p>
    <h1>Behavior — {actionCount()} public actions</h1>
    <p token="doc-lead">
      <code>use:action</code> attaches reusable, attach-anywhere behavior.
      This table is the curated public surface. Widget-private wiring
      (tabs, menu, pagination, radio-group, combobox) lives as closed
      implementation inside the L1 widget — use the tag, not a public
      <code>use:</code>. Author actions live under <code>src/actions/</code>
      under a strict compile floor.
    </p>
  </column>

  <grid cols="3" gap="2">
    <for {actions()} as="a" key={a.name}>
      <card padding="sm">
        style.minW(0)

        <column gap="1">
          <row gap="2" align="center">
            <code>use:{a.name}</code>
            <small>
              style.fg(muted-foreground)
{a.param}</small>
          </row>
          <small>
            style.fg(muted-foreground).maxW(panel-max-w-sm)
{a.summary}</small>
        </column>
      </card>
    </for>
  </grid>

  <row>
    <a variant="default" href="/sail/reference/motion">← Motion</a>
    <a variant="solid" href="/sail/reference/template">Template →</a>
  </row>
</column>
