<!--
  /sail/reference/layout — the layout keyword vocabulary, rendered from
  src/language-surface.gen.ts (emitted by `sail-compiler prop-registry`).
-->
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { LAYOUT_KEYWORDS } from '../../../../src/language-surface.gen'

const LAYOUT_ROWS = LAYOUT_KEYWORDS.map((k) => ({ tag: k.tag, props: k.props.join(', ') }))

export default controller((_ctx: ControllerScope<Scope>) => ({
  layoutRows: () => LAYOUT_ROWS,
}))
</controller>

<column gap="6">
  style.minW(0)

  <column gap="5">
    style.maxW(breakpoint-tablet)

    <p token="doc-eyebrow">Reference</p>
    <h1>Layout keywords</h1>
    <p token="doc-lead">
      Structure is elements, not CSS. Each keyword takes a closed set of
      axis props; anything else is a compile error.
    </p>
  </column>

  <grid cols="3" gap="2">
    <for {layoutRows()} as="k" key={k.tag}>
      <card padding="sm">
        style.minW(0)

        <column gap="1">
          <code>&lt;{k.tag}&gt;</code>
          <small>
            style.fg(muted-foreground).maxW(panel-max-w-sm)
{k.props}</small>
        </column>
      </card>
    </for>
  </grid>

  <row>
    <a variant="default" href="/sail/reference/elements">← Elements</a>
    <a variant="solid" href="/sail/reference/sx">sx →</a>
  </row>
</column>
