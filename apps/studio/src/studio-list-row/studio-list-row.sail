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
  style rowRoot = { size({ h: 6, min-w: 0 }).pad({ x: 3 }), when: { hover: bg(hover-bg), selected: bg(surface-2) } }
  style titleText = { grow(1).size({ min-w: 0 }).type({ size: sm, wrap: truncate }) }
  style metaText = { shrink(0).fg(muted-foreground).type({ size: xs }) }
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
