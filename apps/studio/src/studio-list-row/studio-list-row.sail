<!--
  <studio-list-row> — dense IDE list row: leading · truncate title · meta.
  Props: title, meta, selected. Slots: leading (badge), trailing, default.
  Selected: aria-selected drives when({ selected }) paint.
  Ban-clean: layout keywords + L1 only.
-->
<scope>
  props: {
    title = ''
    meta = ''
    selected = false
  }
</scope>

<row
  studio-list-row
  selected={selected() ? 'true' : 'false'}
  aria-selected={selected() ? 'true' : 'false'}
  gap="2"
  align="center"
  nowrap
  min-w="0"
  sx={style.h(6).padX(3).minW(0).when({ hover: style.bg(hover-bg), selected: style.bg(surface-2) })}
>
  <slot name="leading"></slot>
  <p grow min-w="0" sx={style.fontSize(sm).truncate()}>{title()}</p>
  <if {meta()}>
    <p studio-list-meta shrink="0" sx={style.fontSize(xs).fg(muted-foreground)}>
      {meta()}
    </p>
  </if>
  <slot name="trailing"></slot>
  <slot></slot>
</row>
