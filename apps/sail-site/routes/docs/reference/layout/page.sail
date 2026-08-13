<!--
  /docs/reference/layout — layout keyword vocabulary from language-surface.
-->
<scope>
  style root = { box: size({ min-w: 0 }) }
  style intro = { box: size({ max-w: breakpoint-tablet }) }
  style eyebrow = { paint: ink({ color: accent }), text: type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style lead = { paint: ink({ color: muted-foreground }), text: type({ size: lg, leading: relaxed }) }
  style cardMin = { box: size({ min-w: 0 }) }
  style propsLine = { box: size({ max-w: panel-max-w-sm }), paint: ink({ color: muted-foreground }) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { LAYOUT_KEYWORDS } from '../../../../src/compositions/language-surface'

const LAYOUT_ROWS = LAYOUT_KEYWORDS.map((k) => ({ tag: k.tag, props: k.props.join(', ') }))

export default controller((_ctx: ControllerScope<Scope>) => ({
  layoutRows: () => LAYOUT_ROWS,
}))
</controller>

<column gap="6" style(root)>
  <column gap="5" style(intro)>
    <p style(eyebrow)>Reference</p>
    <h1>Layout keywords</h1>
    <p style(lead)>
      Structure is elements, not CSS. Each keyword takes a closed set of
      axis props; anything else is a compile error.
    </p>
  </column>

  <grid cols="3" gap="2">
    <for {layoutRows() as k} key={k.tag}>
      <card padding="sm" style(cardMin)>
        <column gap="1">
          <code>&lt;{k.tag}&gt;</code>
          <small style(propsLine)>{k.props}</small>
        </column>
      </card>
    </for>
  </grid>

  <row>
    <a variant="default" href="/docs/reference/elements">← Elements</a>
    <a variant="solid" href="/docs/reference/sx">Style →</a>
  </row>
</column>
