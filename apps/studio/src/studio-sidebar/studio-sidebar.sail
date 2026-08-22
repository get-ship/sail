<!--
  <studio-sidebar> — primary side bar: header + optional toolbar + flex body.
  Fixed width 64; header height 8. Body is size(grow) — scroll lives in the body slot.
  Slots: header, toolbar, default.
-->
<scope>
  props: {
    title = ''
  }
  // no preset: size lock + edge-only border
  style sidebarRoot = { size({ w: 64, h: 100%, min-w: 64, min-h: 0 }).overflow(hidden).border(default-r).bg(surface) }
  // no preset: size lock + edge-only border
  style sidebarHeader = { size({ h: 8, min-h: 8 }).pad(3).border(subtle-b) }
  style eyebrow = { fg(muted).type(label) }
  style sidebarBody = { size(grow).overflow(hidden) }
</scope>

<column studio-sidebar gap="0" style(sidebarRoot)>
  <slot name="header">
    <row studio-sidebar-header align="center" style(sidebarHeader)>
      <p style(eyebrow)>
        {title()}
      </p>
    </row>
  </slot>
  <slot name="toolbar"></slot>
  <column studio-sidebar-body gap="0" style(sidebarBody)>
    <slot></slot>
  </column>
</column>
