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
  // no preset: min-w lock
  style rowRoot = { size({ h: 6, min-w: 0 }).pad(3), when: { hover: bg(hover-bg), selected: bg(surface) } }
  style titleText = { size(grow).type(body-sm) }
  style metaText = { size(hug).fg(muted).type(body-sm) }
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
