<!--
  /docs/reference/elements — every stdlib L1 element from prop-registry gen.
-->
<scope>
  style root = { box: size({ min-w: 0 }) }
  style intro = { box: size({ max-w: breakpoint-tablet }) }
  style eyebrow = { paint: ink({ color: accent }), text: type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style lead = { paint: ink({ color: muted-foreground }), text: type({ size: lg, leading: relaxed }) }
  style cardMin = { box: size({ min-w: 0 }) }
  style propCount = { paint: ink({ color: muted-foreground }) }
  style cardDoc = { box: size({ max-w: panel-max-w-sm }), paint: ink({ color: muted-foreground }) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { PROP_REGISTRY } from '../../../../src/compositions/prop-registry'
import { ELEMENTS } from '../../../../src/compositions/language-surface'

interface ElementRow {
  tag: string
  href: string
  propCount: number
  description: string
}

function elementRows(docs: { tag: string; description: string }[]): ElementRow[] {
  return docs.map((d) => ({
    tag: d.tag,
    href: `/components/${d.tag}`,
    propCount: PROP_REGISTRY[d.tag]?.length ?? 0,
    description: d.description,
  }))
}

const STDLIB_ELEMENTS = elementRows(ELEMENTS.stdlib)

export default controller((_ctx: ControllerScope<Scope>) => ({
  stdlibElements: () => STDLIB_ELEMENTS,
  stdlibCount: () => STDLIB_ELEMENTS.length,
}))
</controller>

<column gap="6" style(root)>
  <column gap="5" style(intro)>
    <p style(eyebrow)>Reference</p>
    <h1>Elements — {stdlibCount()} stdlib</h1>
    <p style(lead)>
      Every standard element, with its one-line description and its full
      prop table on its detail page. Dividers are <code>separator</code>;
      keycaps are <code>kbd</code>; disclosure is <code>accordion</code>.
    </p>
  </column>

  <column gap="3">
    <grid cols="3" gap="2">
      <for {stdlibElements() as e} key={e.tag}>
        <card padding="sm" style(cardMin)>
          <column gap="1">
            <a href={e.href}><code>{e.tag}</code> <small style(propCount)>{e.propCount}</small></a>
            <small style(cardDoc)>{e.description}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <row>
    <a variant="default" href="/docs/reference">← Reference index</a>
    <a variant="solid" href="/docs/reference/layout">Layout keywords →</a>
  </row>
</column>
