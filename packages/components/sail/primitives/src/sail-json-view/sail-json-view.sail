<!--
  <sail-json-view> — JSON tree. `data` is a JSON string; `depth` auto-opens
  levels below it; `expanded` opens everything. Branch rows toggle on
  click/Enter/Space and carry aria-expanded; the host is role=tree and rows
  are role=treeitem with aria-level. Invalid JSON renders a role=alert row.
  Level guides are per-level gutter spans with a subtle inline-start border.
-->

<scope>
  props: {
    data = ''
    expanded = false
    depth = 1
  }
</scope>

<controller>

import type { Scope } from './sail-json-view.sail'

type JsonValue = null | boolean | number | string | JsonValue[] | { [k: string]: JsonValue }

interface Pad {
  i: number
}

interface Row {
  k: string
  path: string
  kind: 'branch' | 'close' | 'str' | 'num' | 'bool' | 'null' | 'empty'
  level: number
  ariaLevel: string
  pads: Pad[]
  label: string
  text: string
  open: boolean
  openBracket: string
  closeBracket: string
  meta: string
}

export default controller(({ data, expanded, depth }: Scope) => {
    // Per-path user toggles win over the depth/expanded defaults.
    const overrides = signal<Record<string, boolean>>({})

    const parseError = derived((): string => {
      const d = data()
      if (!d) return ''
      try {
        JSON.parse(d)
        return ''
      } catch (e) {
        return (e as Error).message
      }
    })

    const rows = derived((): Row[] => {
      const d = data()
      if (!d || parseError() !== '') return []
      const parsed = JSON.parse(d) as JsonValue
      const out: Row[] = []
      const ov = overrides()
      const exp = expanded()
      const dep = depth()

      const pads = (level: number): Pad[] =>
        Array.from({ length: level }, (_, i) => ({ i }))

      const walk = (v: JsonValue, label: string, level: number, path: string): void => {
        const base = { path, level, ariaLevel: String(level + 1), pads: pads(level), label, meta: '', open: false, openBracket: '', closeBracket: '' }
        if (v === null) {
          out.push({ ...base, k: `${path}:null`, kind: 'null', text: 'null' })
          return
        }
        if (typeof v === 'string') {
          out.push({ ...base, k: `${path}:str:${v}`, kind: 'str', text: `"${v}"` })
          return
        }
        if (typeof v === 'number') {
          out.push({ ...base, k: `${path}:num:${v}`, kind: 'num', text: String(v) })
          return
        }
        if (typeof v === 'boolean') {
          out.push({ ...base, k: `${path}:bool:${v}`, kind: 'bool', text: String(v) })
          return
        }
        const isArr = Array.isArray(v)
        const entries: Array<[string, JsonValue]> = isArr
          ? (v as JsonValue[]).map((x, i) => [String(i), x])
          : Object.entries(v as Record<string, JsonValue>)
        const openBracket = isArr ? '[' : '{'
        const closeBracket = isArr ? ']' : '}'
        if (entries.length === 0) {
          out.push({ ...base, k: `${path}:empty`, kind: 'empty', text: `${openBracket}${closeBracket}` })
          return
        }
        const open = ov[path] ?? (exp || level < dep)
        out.push({
          ...base,
          k: `${path}:branch:${open}`,
          kind: 'branch',
          text: '',
          open,
          openBracket,
          closeBracket,
          meta: `${entries.length} ${isArr ? 'items' : 'keys'}`,
        })
        if (open) {
          for (const [k, val] of entries) {
            walk(val, isArr ? '' : k, level + 1, `${path}.${k}`)
          }
          out.push({ ...base, k: `${path}:close`, kind: 'close', text: closeBracket })
        }
      }

      walk(parsed, '', 0, '$')
      return out
    })

    const toggle = (path: string, open: boolean): void => {
      overrides.set((cur) => ({ ...cur, [path]: !open }))
    }

    mount((host) => {
      if (!host.hasAttribute('role')) host.setAttribute('role', 'tree')
      if (!host.hasAttribute('aria-label')) host.setAttribute('aria-label', 'JSON tree')
    })

    return { rows, parseError, toggle }
  })
</controller>

<column gap="0">
  style.fontFamily(mono).fontSize(sm).leading(normal-plus)

  <if {parseError() !== ''}>
    <span role="alert">
      style.fg(destructive)
Invalid JSON: {parseError()}</span>
  </if>
  <for {rows()} as="row" key={row.k}>
    <match {row.kind}>
      <case {'branch'}>
        <row role="treeitem" aria-level={row.ariaLevel} aria-expanded={row.open ? 'true' : 'false'} tabindex="0" gap="0" align="center"
             on:click={() => toggle(row.path, row.open)}
             on:keydown|key:Enter={() => toggle(row.path, row.open)}
>
          style.group().radius(sm).cursor(pointer).when({ hover: style.bg(hover-bg) })

          <for {row.pads} as="pad" key={pad.i}>
            <span>
              style.w(3).borderLWidth(1).borderL(border-subtle)
</span>
          </for>
          <span aria-hidden="true">
            style.fg(muted-foreground).fontSize(xs).padX(1).when({ groupExpanded: style.rotate(90) })
▶</span>
          <if {row.label !== ''}>
            <span>
              style.fontWeight(medium)
"{row.label}"</span><span>
              style.fg(muted-foreground)
:&nbsp;</span>
          </if>
          <span>
            style.fg(muted-foreground)
{row.openBracket}</span>
          <span>
            style.fg(muted-foreground).fontSize(xs).bg(muted).radius(full).padX(1).ml(1)
{row.meta}</span>
          <if {!row.open}>
            <span>
              style.fg(muted-foreground)
…{row.closeBracket}</span>
          </if>
        </row>
      </case><case {'close'}>
        <row gap="0" align="center">
          <for {row.pads} as="pad" key={pad.i}>
            <span>
              style.w(3).borderLWidth(1).borderL(border-subtle)
</span>
          </for>
          <span>
            style.fg(muted-foreground).padX(1)
{row.text}</span>
        </row>
      </case><default>
        <row role="treeitem" aria-level={row.ariaLevel} gap="0" align="center">
          style.radius(sm).when({ hover: style.bg(hover-bg) })

          <for {row.pads} as="pad" key={pad.i}>
            <span>
              style.w(3).borderLWidth(1).borderL(border-subtle)
</span>
          </for>
          <if {row.label !== ''}>
            <span>
              style.fontWeight(medium).padX(1)
"{row.label}"</span><span>
              style.fg(muted-foreground)
:&nbsp;</span>
          </if>
          <match {row.kind}>
            <case {'str'}> <span>
              style.fg(success)
{row.text}</span>
            </case><case {'num'}> <span>
              style.fg(info)
{row.text}</span>
            </case><case {'bool'}> <span>
              style.fg(accent).fontWeight(medium)
{row.text}</span>
            </case><case {'null'}> <span>
              style.fg(muted-foreground)
{row.text}</span>
            </case><default> <span>
              style.fg(muted-foreground)
{row.text}</span>
          </default></match>
        </row>
    </default></match>
  </for>
</column>
