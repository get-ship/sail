<!--
  <studio-sidebar> — primary side bar: header + optional toolbar + flex body.
  Fixed width 64; header height 8. Body is grow + min-h 0 — scroll lives in the body slot.
  Slots: header, toolbar, default.
-->
<scope>
  props: {
    title = ''
  }
  style sidebarRoot = { box: flow({ shrink: 0 }).size({ w: 64, h: full, min-w: 64, min-h: 0 }).overflow({ all: hidden }), paint: border({ edge: r, color: border, width: 1 }).fill({ color: surface-1 }) }
  style sidebarHeader = { box: flow({ shrink: 0 }).size({ h: 8, min-h: 8 }).space({ pad-x: 3 }), paint: border({ edge: b, color: border-subtle, width: 1 }) }
  style eyebrow = { paint: ink({ color: muted-foreground }), text: type({ size: xs, weight: semibold, tracking: wide, case: uppercase }) }
  style sidebarBody = { box: flow({ grow: 1 }).size({h: full, min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }) }
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
