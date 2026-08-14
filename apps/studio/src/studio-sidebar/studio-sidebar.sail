<!--
  <studio-sidebar> — primary side bar: header + optional toolbar + flex body.
  Fixed width 64; header height 8. Body is grow + min-h 0 — scroll lives in the body slot.
  Slots: header, toolbar, default.
-->
<scope>
  props: {
    title = ''
  }
  style sidebarRoot = { shrink(0).size({ w: 64, h: full, min-w: 64, min-h: 0 }).overflow({ all: hidden }).border({ edge: r, color: border, width: 1 }).bg(surface-1) }
  style sidebarHeader = { shrink(0).size({ h: 8, min-h: 8 }).pad({ x: 3 }).border({ edge: b, color: border-subtle, width: 1 }) }
  style eyebrow = { fg(muted-foreground).type({ size: xs, weight: semibold, tracking: wide, case: uppercase }) }
  style sidebarBody = { grow(1).size({h: full, min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }) }
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
