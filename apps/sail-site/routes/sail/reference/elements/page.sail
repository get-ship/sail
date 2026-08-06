<!--
  /sail/reference/elements — every stdlib element + native reskin, rendered
  from src/prop-registry.gen.ts + src/language-surface.gen.ts (emitted by
  `sail-compiler prop-registry`). Nothing here is hand-listed.
-->
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { PROP_REGISTRY } from '../../../../src/prop-registry.gen'
import { ELEMENTS } from '../../../../src/language-surface.gen'

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
const RESKIN_ELEMENTS = elementRows(ELEMENTS.reskins)

export default controller((_ctx: ControllerScope<Scope>) => ({
  stdlibElements: () => STDLIB_ELEMENTS,
  stdlibCount: () => STDLIB_ELEMENTS.length,
  reskinElements: () => RESKIN_ELEMENTS,
  reskinCount: () => RESKIN_ELEMENTS.length,
}))
</controller>

<column gap="6">
  style.minW(0)

  <column gap="5">
    style.maxW(breakpoint-tablet)

    <p token="doc-eyebrow">Reference</p>
    <h1>Elements — {stdlibCount()} stdlib + {reskinCount()} reskins</h1>
    <p token="doc-lead">
      Every standard element, with its one-line description and its full
      prop table on its detail page.
    </p>
  </column>

  <column gap="3">
    <grid cols="3" gap="2">
      <for {stdlibElements()} as="e" key={e.tag}>
        <card padding="sm">
          style.minW(0)

          <column gap="1">
            <a href={e.href}><code>{e.tag}</code> <small>
              style.fg(muted-foreground)
{e.propCount}</small></a>
            <small>
              style.fg(muted-foreground).maxW(panel-max-w-sm)
{e.description}</small>
          </column>
        </card>
      </for>
    </grid>

    <h3>Native reskins</h3>
    <p>
      style.maxW(breakpoint-tablet).fontSize(sm).fg(muted-foreground)

      Runtime-styled native HTML tags (<code>hr</code>, <code>kbd</code>).
      The tag remains the API; the framework only reskins paint. Prefer
      stdlib <code>separator</code> when you need orientation, variant, or
      tone props.
    </p>
    <grid cols="3" gap="2">
      <for {reskinElements()} as="e" key={e.tag}>
        <card padding="sm">
          style.minW(0)

          <column gap="1">
            <a href={e.href}><code>{e.tag}</code></a>
            <small>
              style.fg(muted-foreground).maxW(panel-max-w-sm)
{e.description}</small>
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
