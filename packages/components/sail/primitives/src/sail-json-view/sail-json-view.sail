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
  style guidePad = { box: size({ w: 3 }), paint: border({ edge: l, color: border-subtle, width: 1 }) }
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
  <span style({ box: flow({ display: contents }), text: type({ family: mono, size: sm, leading: loose }) })>
    <if {parseError() !== ''}>
      <span role="alert" style({ paint: ink({ color: destructive }) })>Invalid JSON: {parseError()}</span>
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
          style({ paint: group().radius(sm).cursor(pointer), when: { hover: { paint: fill({ color: hover-bg }) } } })
        >
          <for {row.pads as pad} key={pad.i}>
            <span style(guidePad)></span>
          </for>
          <span
            aria-hidden="true"
            style({
              box: space({ pad-x: 1 }),
              paint: ink({ color: muted-foreground }),
              text: type({ size: xs }),
              when: { groupExpanded: { paint: motion({ rotate: 90 }) } },
            })
          >▶</span>
          <if {row.label !== ''}>
            <span style({ text: type({ weight: medium }) })>"{row.label}"</span><span style({ paint: ink({ color: muted-foreground }) })>:&nbsp;</span>
          </if>
          <span style({ paint: ink({ color: muted-foreground }) })>{row.openBracket}</span>
          <span style({ box: space({ pad-x: 1, ml: 1 }), paint: ink({ color: muted-foreground }).fill({ color: muted }).radius(full), text: type({ size: xs }) })>{row.meta}</span>
          <if {!row.open}>
            <span style({ paint: ink({ color: muted-foreground }) })>…{row.closeBracket}</span>
          </if>
        </row>}{'close' => <row gap="0" align="center">
          <for {row.pads as pad} key={pad.i}>
            <span style(guidePad)></span>
          </for>
          <span style({ box: space({ pad-x: 1 }), paint: ink({ color: muted-foreground }) })>{row.text}</span>
        </row>}{else => <row
          role="treeitem"
          aria-level={row.ariaLevel}
          gap="0"
          align="center"
          style({ paint: radius(sm), when: { hover: { paint: fill({ color: hover-bg }) } } })
        >
          <for {row.pads as pad} key={pad.i}>
            <span style(guidePad)></span>
          </for>
          <if {row.label !== ''}>
            <span style({ box: space({ pad-x: 1 }), text: type({ weight: medium }) })>"{row.label}"</span><span style({ paint: ink({ color: muted-foreground }) })>:&nbsp;</span>
          </if>
          <match {row.kind}>
            {'str' => <span style({ paint: ink({ color: success }) })>{row.text}</span>}{'num' => <span style({ paint: ink({ color: info }) })>{row.text}</span>}{'bool' => <span style({ paint: ink({ color: accent }), text: type({ weight: medium }) })>{row.text}</span>}{'null' => <span style({ paint: ink({ color: muted-foreground }) })>{row.text}</span>}{else => <span style({ paint: ink({ color: muted-foreground }) })>{row.text}</span>}</match>
        </row>}</match>
    </for>
  </span>
</column>
