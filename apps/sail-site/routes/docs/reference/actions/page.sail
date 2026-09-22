<!--
  /docs/reference/actions — public use: registry from language-surface gen.
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { PUBLIC_ACTIONS } from '../../../../src/compositions/language-surface'

export default controller((_ctx: ControllerScope<Scope>) => ({
  actions: () => PUBLIC_ACTIONS,
  actionCount: () => PUBLIC_ACTIONS.length,
}))
</controller>

<column gap="6">
  <column gap="5">
    <p>Reference</p>
    <h1>Behavior — {actionCount()} public actions</h1>
    <p>
      <code>use:action</code> attaches reusable, attach-anywhere behavior.
      This table is the curated public surface. Widget-private wiring
      (tabs, menu, radio-group, combobox) lives as closed
      implementation inside the L1 widget — use the tag, not a public
      <code>use:</code>. Author actions live under <code>src/actions/</code>
      under a strict compile floor.
    </p>
  </column>

  <grid cols="3" gap="2">
    <for {actions() as a} key={a.name}>
      <card padding="sm">
        <column gap="1">
          <row gap="2" align="center">
            <code>use:{a.name}</code>
            <small>{a.param}</small>
          </row>
          <small>{a.summary}</small>
        </column>
      </card>
    </for>
  </grid>

  <row>
    <a variant="default" href="/docs/reference/motion">← Motion</a>
    <a variant="solid" href="/docs/reference/template">Template →</a>
  </row>
</column>
