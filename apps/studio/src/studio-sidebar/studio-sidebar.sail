<!--
  <studio-sidebar> — primary side bar: header + optional toolbar + flex body.
  Fixed w(64); header h(8). Body is grow + minH(0) — scroll lives in the body slot.
  Slots: header, toolbar, default.
-->
<scope>
  props: {
    title = ''
  }
  style sidebarRoot = { box: shrink(0).w(64).minW(64).minH(0).h(full).overflow(hidden), paint: borderRWidth(1).borderR(border).bg(surface-1) }
  style sidebarHeader = { box: shrink(0).h(8).minH(8).padX(3), paint: borderBWidth(1).borderB(border-subtle) }
  style eyebrow = { paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide) }
  style sidebarBody = { box: grow(1).minW(0).minH(0).h(full).basis(0).overflow(hidden) }
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
