<!--
  <studio-list-row> — dense IDE list row: leading · truncate title · meta.
  Props: title, meta, selected. Slots: leading (badge), trailing, default.
  Selected: selected dual-stamps aria-selected for when({ selected }) paint.
  Ban-clean: layout keywords + L1 only.
-->
<scope>
  props: {
    title = ''
    meta = ''
    selected = false
  }
</scope>

<row studio-list-row selected={selected() ? 'true' : 'false'} gap="2" align="center" nowrap style({  box: minW(0).h(6).padX(3)  }).when(hover, { paint: bg(hover-bg) }).when(selected, { paint: bg(surface-2) })>
  <slot name="leading"></slot>
  <p style({ box: grow(1).minW(0), text: fontSize(sm).truncate() })>{title()}</p>
  <if {meta()}>
    <p studio-list-meta style({ box: shrink(0), paint: fg(muted-foreground), text: fontSize(xs) })>
      {meta()}
    </p>
  </if>
  <slot name="trailing"></slot>
  <slot></slot>
</row>
