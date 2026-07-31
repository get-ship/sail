<!--
  /sail/reference/actions — the use: registry, rendered from
  src/language-surface.gen.ts (emitted by `sail-compiler prop-registry`).
-->
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { ACTIONS } from '../../../../src/language-surface.gen'

export default controller((_ctx: ControllerScope<Scope>) => ({
  actions: () => ACTIONS,
  actionCount: () => ACTIONS.length,
}))
</controller>

<column gap="6" sx={style.minW(0)}>
  <column sx={style.maxW(breakpoint-tablet)} gap="5">
    <p role="doc-eyebrow">Reference</p>
    <h1>Behavior — {actionCount()} actions</h1>
    <p role="doc-lead">
      <code>use:action</code> attaches behavior bound to an element — the
      built-in <code>use:</code> registry, sorted, param + one-line summary.
      A scope value works the same way for a one-off, non-built-in action.
    </p>
  </column>

  <grid cols="3" gap="2">
    <for {actions()} as="a" key={a.name}>
      <card padding="sm" sx={style.minW(0)}>
        <column gap="1">
          <row gap="2" align="center">
            <code>use:{a.name}</code>
            <small sx={style.fg(muted-foreground)}>{a.param}</small>
          </row>
          <small sx={style.fg(muted-foreground).maxW(panel-max-w-sm)}>{a.summary}</small>
        </column>
      </card>
    </for>
  </grid>

  <row role="navigation">
    <a variant="default" href="/sail/reference/motion">← Motion</a>
    <a variant="solid" href="/sail/reference/template">Template →</a>
  </row>
</column>
