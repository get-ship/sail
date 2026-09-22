<!--
  /docs/reference/elements — every stdlib L1 element from prop-registry gen.
-->

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

<column gap="6">
  <column gap="5">
    <p>Reference</p>
    <h1>Elements — {stdlibCount()} stdlib</h1>
    <p>
      Every standard element, with its one-line description and its full
      prop table on its detail page. Dividers are <code>separator</code>;
      keycaps are <code>kbd</code>; disclosure is <code>accordion</code>.
    </p>
  </column>

  <column gap="3">
    <grid cols="3" gap="2">
      <for {stdlibElements() as e} key={e.tag}>
        <card padding="sm">
          <column gap="1">
            <a href={e.href}><code>{e.tag}</code> <small>{e.propCount}</small></a>
            <small>{e.description}</small>
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
