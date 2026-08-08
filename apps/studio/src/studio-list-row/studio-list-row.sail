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
    box: minW(0).h(6).padX(3),
    when: {
      hover: { paint: bg(hover-bg) },
      selected: { paint: bg(surface-2) },
    },
  }
  style titleText = { box: grow(1).minW(0), text: fontSize(sm).truncate() }
  style metaText = { box: shrink(0), paint: fg(muted-foreground), text: fontSize(xs) }
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
