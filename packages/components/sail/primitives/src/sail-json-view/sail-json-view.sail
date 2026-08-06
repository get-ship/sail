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
  <span>
    style({ box: display(contents), text: fontFamily(mono).fontSize(sm).leading(loose) })
    <if {parseError() !== ''}>
    <span role="alert">
      style({ paint: fg(destructive) })
Invalid JSON: {parseError()}</span>
  </if>
  <for {rows() as row} key={row.k}>
    <match {row.kind}>
      {'branch' => <row role="treeitem" aria-level={row.ariaLevel} aria-expanded={row.open ? 'true' : 'false'} tabindex="0" gap="0" align="center"
             on:click={() => toggle(row.path, row.open)}
             on:keydown|key:Enter={() => toggle(row.path, row.open)}
>
          style({ paint: group().radius(sm).cursor(pointer) }).when(hover, { paint: bg(hover-bg) })

          <for {row.pads as pad} key={pad.i}>
            <span>
              style({ box: w(3), paint: borderLWidth(1).borderL(border-subtle) })
</span>
          </for>
          <span aria-hidden="true">
            style({ box: padX(1), paint: fg(muted-foreground), text: fontSize(xs) }).when(groupExpanded, { paint: rotate(90) })
▶</span>
          <if {row.label !== ''}>
            <span>
              style({ text: fontWeight(medium) })
"{row.label}"</span><span>
              style({ paint: fg(muted-foreground) })
:&nbsp;</span>
          </if>
          <span>
            style({ paint: fg(muted-foreground) })
{row.openBracket}</span>
          <span>
            style({ box: padX(1).ml(1), paint: fg(muted-foreground).bg(muted).radius(full), text: fontSize(xs) })
{row.meta}</span>
          <if {!row.open}>
            <span>
              style({ paint: fg(muted-foreground) })
…{row.closeBracket}</span>
          </if>
        </row>}{'close' => <row gap="0" align="center">
          <for {row.pads as pad} key={pad.i}>
            <span>
              style({ box: w(3), paint: borderLWidth(1).borderL(border-subtle) })
</span>
          </for>
          <span>
            style({ box: padX(1), paint: fg(muted-foreground) })
{row.text}</span>
        </row>}{else => <row role="treeitem" aria-level={row.ariaLevel} gap="0" align="center">
          style({ paint: radius(sm) }).when(hover, { paint: bg(hover-bg) })

          <for {row.pads as pad} key={pad.i}>
            <span>
              style({ box: w(3), paint: borderLWidth(1).borderL(border-subtle) })
</span>
          </for>
          <if {row.label !== ''}>
            <span>
              style({ box: padX(1), text: fontWeight(medium) })
"{row.label}"</span><span>
              style({ paint: fg(muted-foreground) })
:&nbsp;</span>
          </if>
          <match {row.kind}>
            {'str' => <span>
              style({ paint: fg(success) })
{row.text}</span>}{'num' => <span>
              style({ paint: fg(info) })
{row.text}</span>}{'bool' => <span>
              style({ paint: fg(accent), text: fontWeight(medium) })
{row.text}</span>}{'null' => <span>
              style({ paint: fg(muted-foreground) })
{row.text}</span>}{else => <span>
              style({ paint: fg(muted-foreground) })
{row.text}</span>}</match>
        </row>}</match>
  </for>
  </span>
</column>
