<!--
  /sail/reference/elements — every stdlib L1 element, rendered from
  src/prop-registry.gen.ts + src/language-surface.gen.ts (emitted by
  `sail-compiler prop-registry`). Nothing here is hand-listed.
  Native reskins retired (SPEC-L1-1).
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

<column gap="6" style({ box: minW(0) })>
  <column gap="5" style({ box: maxW(breakpoint-tablet) })>
    <p token="doc-eyebrow">Reference</p>
    <h1>Elements — {stdlibCount()} stdlib</h1>
    <p token="doc-lead">
      Every standard element, with its one-line description and its full
      prop table on its detail page. Dividers are <code>separator</code>;
      keycaps are <code>kbd</code>; disclosure is <code>accordion</code>.
    </p>
  </column>

  <column gap="3">
    <grid cols="3" gap="2">
      <for {stdlibElements() as e} key={e.tag}>
        <card padding="sm" style({ box: minW(0) })>
          <column gap="1">
            <a href={e.href}><code>{e.tag}</code> <small style({ paint: fg(muted-foreground) })>{e.propCount}</small></a>
            <small style({ box: maxW(panel-max-w-sm), paint: fg(muted-foreground) })>{e.description}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <row>
    <a variant="default" href="/sail/reference">← Reference index</a>
    <a variant="solid" href="/sail/reference/layout">Layout keywords →</a>
  </row>
</column>
