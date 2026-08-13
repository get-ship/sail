<!--
  <studio-list-row> — dense IDE list row: leading · truncate title · meta.
  Props: title, meta, selected. Slots: leading, trailing, default.
  selected dual-stamps aria-selected for when:{selected} paint.
-->
<scope>
  props: {
    title = ''
    meta = ''
    selected = false
  }
  style rowRoot = {
    box: size({ h: 6, min-w: 0 }).space({ pad-x: 3 }),
    when: {
      hover: { paint: fill({ color: hover-bg }) },
      selected: { paint: fill({ color: surface-2 }) },
    },
  }
  style titleText = { box: flow({ grow: 1 }).size({ min-w: 0 }), text: type({ size: sm, wrap: truncate }) }
  style metaText = { box: flow({ shrink: 0 }), paint: ink({ color: muted-foreground }), text: type({ size: xs }) }
</scope>

<row studio-list-row selected={selected() ? 'true' : 'false'} gap="2" align="center" nowrap style(rowRoot)>
  <slot name="leading"></slot>
  <p style(titleText)>{title()}</p>
  <if {meta()}>
    <p studio-list-meta style(metaText)>
      {meta()}
    </p>
  </if>
  <slot name="trailing"></slot>
  <slot></slot>
</row>
