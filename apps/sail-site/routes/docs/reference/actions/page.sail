<!--
  /docs/reference/actions — public use: registry from language-surface gen.
-->
<scope>
  style root = { box: minW(0) }
  style intro = { box: maxW(breakpoint-tablet) }
  style eyebrow = { paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) }
  style cardMin = { box: minW(0) }
  style param = { paint: fg(muted-foreground) }
  style cardDoc = { box: maxW(panel-max-w-sm), paint: fg(muted-foreground) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { PUBLIC_ACTIONS } from '../../../../src/compositions/language-surface'

export default controller((_ctx: ControllerScope<Scope>) => ({
  actions: () => PUBLIC_ACTIONS,
  actionCount: () => PUBLIC_ACTIONS.length,
}))
</controller>

<column gap="6" style(root)>
  <column gap="5" style(intro)>
    <p style(eyebrow)>Reference</p>
    <h1>Behavior — {actionCount()} public actions</h1>
    <p style(lead)>
      <code>use:action</code> attaches reusable, attach-anywhere behavior.
      This table is the curated public surface. Widget-private wiring
      (tabs, menu, pagination, radio-group, combobox) lives as closed
      implementation inside the L1 widget — use the tag, not a public
      <code>use:</code>. Author actions live under <code>src/actions/</code>
      under a strict compile floor.
    </p>
  </column>

  <grid cols="3" gap="2">
    <for {actions() as a} key={a.name}>
      <card padding="sm" style(cardMin)>
        <column gap="1">
          <row gap="2" align="center">
            <code>use:{a.name}</code>
            <small style(param)>{a.param}</small>
          </row>
          <small style(cardDoc)>{a.summary}</small>
        </column>
      </card>
    </for>
  </grid>

  <row>
    <a variant="default" href="/docs/reference/motion">← Motion</a>
    <a variant="solid" href="/docs/reference/template">Template →</a>
  </row>
</column>
