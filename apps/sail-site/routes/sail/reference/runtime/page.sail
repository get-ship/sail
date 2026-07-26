<!--
  /sail/reference/runtime — the complete public export surface of
  @ship/sail-runtime, one section per entry point (package.json `exports`),
  plus the controller's ambient vocabulary. Rendered from
  src/runtime-surface.gen.ts (emitted by scripts/gen-runtime-surface.mjs,
  walked off the TypeScript compiler API — see that script) and
  src/language-surface.gen.ts (CONTROLLER_AMBIENT, from the compiler).
-->
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { RUNTIME_EXPORTS } from '../../../../src/runtime-surface.gen'
import { CONTROLLER_AMBIENT } from '../../../../src/language-surface.gen'

interface SubpathGroup {
  subpath: string
  count: number
  exports: typeof RUNTIME_EXPORTS
}

const BY_SUBPATH = new Map<string, typeof RUNTIME_EXPORTS>()
for (const e of RUNTIME_EXPORTS) {
  const list = BY_SUBPATH.get(e.subpath)
  if (list) {
    list.push(e)
  } else {
    BY_SUBPATH.set(e.subpath, [e])
  }
}
// '.' (the root barrel) first, then subpaths alphabetically.
const SUBPATH_GROUPS: SubpathGroup[] = [...BY_SUBPATH.entries()]
  .sort(([a], [b]) => (a === '.' ? -1 : b === '.' ? 1 : a.localeCompare(b)))
  .map(([subpath, exports]) => ({ subpath, count: exports.length, exports }))

const CONTROLLER_ROWS = CONTROLLER_AMBIENT.filter((a) => a.name !== '__invalidate')

export default controller((_ctx: ControllerScope<Scope>) => ({
  groups: () => SUBPATH_GROUPS,
  totalCount: () => RUNTIME_EXPORTS.length,
  controllerRows: () => CONTROLLER_ROWS,
}))
</controller>

<column gap="6" sx={style.minW(0)}>
  <column sx={style.maxW(breakpoint-tablet)} gap="5">
    <p role="doc-eyebrow">Reference</p>
    <h1>Runtime — {totalCount()} exports</h1>
    <p role="doc-lead">
      Every value and type @ship/sail-runtime exports, from every entry
      point in its package.json <code>exports</code> map. <code>./internal</code>
      is a total re-export of the root barrel (the compiler's own import
      door) and is not listed separately — same surface, one more name.
    </p>
  </column>

  <column gap="3">
    <h2>Controller — the ambient vocabulary</h2>
    <p sx={style.maxW(breakpoint-tablet)}>
      A <code>&lt;controller&gt;</code> block may reference these without
      importing them — the compiler injects the import for exactly the
      idents referenced.
    </p>
    <grid cols="3" gap="2">
      {#for c of controllerRows() key={c.name}}
        <card padding="sm" sx={style.minW(0)}>
          <column gap="1">
            <code>{c.name}</code>
            <small sx={style.fg(muted-foreground).maxW(panel-max-w-sm)}>{c.doc}</small>
          </column>
        </card>
      {/for}
    </grid>
  </column>

  {#for group of groups() key={group.subpath}}
    <column gap="3">
      <h2><code>{group.subpath}</code> — {group.count} exports</h2>
      <column gap="1">
        {#for e of group.exports key={e.name}}
          <row gap="3" sx={style.padX(3).padY(2).radius(md).borderWidth(1).border(border)} align="start">
            <code sx={style.minW(32)}>{e.name}</code>
            <badge size="sm">{e.kind}</badge>
            <column gap="1" sx={style.minW(0)}>
              <code sx={style.fg(muted-foreground).fontFamily(mono)}>{e.signature}</code>
              {#if e.doc}
                <small sx={style.fg(muted-foreground)}>{e.doc}</small>
              {/if}
            </column>
          </row>
        {/for}
      </column>
    </column>
  {/for}

  <row role="navigation">
    <a variant="default" href="/sail/reference/scope">← Scope grammar</a>
    <a variant="solid" href="/sail/reference/config">Config →</a>
  </row>
</column>
