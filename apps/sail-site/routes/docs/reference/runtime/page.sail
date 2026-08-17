<!--
  Author runtime (tier B) — exports from author-surface.json via runtime-surface composition.
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import {
  AUTHOR_RUNTIME_COUNT,
  RUNTIME_CATEGORIES,
  RUNTIME_EXPORTS,
} from '../../../../src/compositions/runtime-surface'
import { CONTROLLER_AMBIENT } from '../../../../src/compositions/language-surface'

interface CategoryGroup {
  id: string
  summary: string
  count: number
  exports: typeof RUNTIME_EXPORTS
}

const BY_CATEGORY = new Map<string, typeof RUNTIME_EXPORTS>()
for (const e of RUNTIME_EXPORTS) {
  const list = BY_CATEGORY.get(e.category)
  if (list) {
    list.push(e)
  } else {
    BY_CATEGORY.set(e.category, [e])
  }
}

const CATEGORY_GROUPS: CategoryGroup[] = RUNTIME_CATEGORIES.map((c) => ({
  id: c.id,
  summary: c.summary,
  count: c.count,
  exports: BY_CATEGORY.get(c.id) ?? [],
}))

const CONTROLLER_ROWS = CONTROLLER_AMBIENT.filter((a) => a.name !== '__invalidate')

export default controller((_ctx: ControllerScope<Scope>) => ({
  groups: () => CATEGORY_GROUPS,
  totalCount: () => AUTHOR_RUNTIME_COUNT,
  controllerRows: () => CONTROLLER_ROWS,
}))
</controller>

<column gap="6">
  <column gap="5">
    <p>Reference · tier B</p>
    <h1>Author runtime — {totalCount()} exports</h1>
    <p>
      Intentional imports from <code>@ship/sail-runtime</code> (root barrel),
      filtered by <code>author-surface.json</code>. Host subpaths
      (<code>./framework</code>, <code>./server</code>, <code>./dev</code>, …)
      and emit-only <code>@ship/sail-runtime/internal</code> (tier C) are not
      listed. Template directives and Style motion live under tier A pages.
    </p>
  </column>

  <column gap="3">
    <h2>Controller — ambient vocabulary (tier A)</h2>
    <p>
      A <code>&lt;controller&gt;</code> block may reference these without
      importing them — the compiler injects the import for exactly the
      idents referenced. Same symbols are also available as tier B imports.
    </p>
    <grid cols="3" gap="2">
      <for {controllerRows() as c} key={c.name}>
        <card padding="sm">
          <column gap="1">
            <code>{c.name}</code>
            <small>{c.doc}</small>
          </column>
        </card>
      </for>
    </grid>
  </column>

  <for {groups() as group} key={group.id}>
    <column gap="3">
      <h2>{group.id} — {group.count}</h2>
      <p>{group.summary}</p>
      <column gap="1">
        <for {group.exports as e} key={e.name}>
          <card padding="sm">
            <row gap="3" align="start">
              <code>{e.name}</code>
              <badge size="sm">{e.kind}</badge>
              <column gap="1">
                <code>{e.signature}</code>
                <if {e.doc}>
                  <small>{e.doc}</small>
                </if>
              </column>
            </row>
          </card>
        </for>
      </column>
    </column>
  </for>

  <row>
    <a variant="default" href="/docs/reference/scope">← Scope grammar</a>
    <a variant="solid" href="/docs/reference/config">Config →</a>
  </row>
</column>
