<!--
  <studio-sidebar> — primary side bar: header + optional toolbar + flex body.
  Fixed width w(64); header h(8) matches activity brand + edit tab strip.
  minH(0)/grow/basis(0)/overflow chain so content swaps do not resize chrome.
  slots: header (overrides title row), toolbar, default (body).
  Body is grow + minH(0) — not a forced scroll-area. Virtualized trees / dual
  panes fill it; lists that scroll put <scroll-area> inside the body slot.
  Ban-clean: layout keywords + L1 only.
-->
<scope>
  props: {
    title = ''
  }
</scope>

<column studio-sidebar gap="0" style({  box: shrink(0).w(64).minW(64).minH(0).h(full).overflow(hidden), paint: borderRWidth(1).borderR(border).bg(surface-1)  })>
  <slot name="header">
    <row studio-sidebar-header align="center" style({  box: shrink(0).h(8).minH(8).padX(3), paint: borderBWidth(1).borderB(border-subtle)  })>
      <p style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide) })>
        {title()}
      </p>
    </row>
  </slot>
  <slot name="toolbar"></slot>
  <column studio-sidebar-body gap="0" style({  box: grow(1).minW(0).minH(0).h(full).basis(0).overflow(hidden)  })>
    <slot></slot>
  </column>
</column>
