<!--
  <sail-json-view> — JSON tree. `data` is a JSON string; `depth` auto-opens
  levels; `expanded` opens all. Branch rows toggle; host role=tree.
-->

<scope>
  props: {
    data = ''
    expanded = false
    depth = 1
  }
  style guidePad = { size({ w: 3 }).border(subtle-l) }
</scope>

<controller>

import type { Scope } from './sail-json-view.sail'
import { buildJsonRows, type JsonValue } from './json-rows'

export default controller(({ data, expanded, depth }: Scope) => {
    // Per-path user toggles win over depth/expanded defaults.
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

    const rows = derived(() => {
      const d = data()
      if (!d || parseError() !== '') return []
      const parsed = JSON.parse(d) as JsonValue
      return buildJsonRows(parsed, overrides(), expanded(), depth())
    })

    const toggle = (path: string, open: boolean): void => {
      overrides.set((cur) => ({ ...cur, [path]: !open }))
    }

    mount((host) => {
      if (!hostHasAttr(host, 'role')) hostSetAttr(host, 'role', 'tree')
      if (!hostHasAttr(host, 'aria-label')) hostSetAttr(host, 'aria-label', 'JSON tree')
    })

    return { rows, parseError, toggle }
  })
</controller>

<column gap="0">
  <span style({ display(contents).type({ family: mono, size: sm, leading: relaxed }) })>
    <if {parseError() !== ''}>
      <span role="alert" style({ fg(danger) })>Invalid JSON: {parseError()}</span>
    </if>
    <for {rows() as row} key={row.k}>
      <match {row.kind}>
        {'branch' => <row
          role="treeitem"
          aria-level={row.ariaLevel}
          aria-expanded={row.open ? 'true' : 'false'}
          tabindex="0"
          gap="0"
          align="center"
          on:click={() => toggle(row.path, row.open)}
          on:keydown|key:Enter={() => toggle(row.path, row.open)}
          style({ group().radius(sm).cursor(pointer), when: { hover: bg(hover-bg) } })
        >
          <for {row.pads as pad} key={pad.i}>
            <span style(guidePad)></span>
          </for>
          <span
            aria-hidden="true"
            style({ pad({ x: 1 }).fg(muted-foreground).type({ size: xs }), when: { groupExpanded: rotate(90) } })
          >▶</span>
          <if {row.label !== ''}>
            <span style({ type({ weight: medium }) })>"{row.label}"</span><span style({ fg(muted-foreground) })>:&nbsp;</span>
          </if>
          <span style({ fg(muted-foreground) })>{row.openBracket}</span>
          <span style({ pad({ x: 1 }).margin({ l: 1 }).fg(muted-foreground).bg(muted).radius(full).type({ size: xs }) })>{row.meta}</span>
          <if {!row.open}>
            <span style({ fg(muted-foreground) })>…{row.closeBracket}</span>
          </if>
        </row>}{'close' => <row gap="0" align="center">
          <for {row.pads as pad} key={pad.i}>
            <span style(guidePad)></span>
          </for>
          <span style({ pad({ x: 1 }).fg(muted-foreground) })>{row.text}</span>
        </row>}{else => <row
          role="treeitem"
          aria-level={row.ariaLevel}
          gap="0"
          align="center"
          style({ radius(sm), when: { hover: bg(hover-bg) } })
        >
          <for {row.pads as pad} key={pad.i}>
            <span style(guidePad)></span>
          </for>
          <if {row.label !== ''}>
            <span style({ pad({ x: 1 }).type({ weight: medium }) })>"{row.label}"</span><span style({ fg(muted-foreground) })>:&nbsp;</span>
          </if>
          <match {row.kind}>
            {'str' => <span style({ fg(success) })>{row.text}</span>}{'num' => <span style({ fg(info) })>{row.text}</span>}{'bool' => <span style({ fg(accent).type({ weight: medium }) })>{row.text}</span>}{'null' => <span style({ fg(muted-foreground) })>{row.text}</span>}{else => <span style({ fg(muted-foreground) })>{row.text}</span>}</match>
        </row>}</match>
    </for>
  </span>
</column>
