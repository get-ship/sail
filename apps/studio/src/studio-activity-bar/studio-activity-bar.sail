<!--
  <studio-activity-bar> — VS Code activity rail.
  Outer w(12); items w/h 10 with padX(1). overflow visible so right tooltips are not clipped.
  Slots: brand, default (items), footer.
-->
<scope>
  style railRoot = { box: shrink(0).display(flex).flexDir(column).w(12).minW(12).maxW(12).minH(0).h(full).overflow(visible), paint: bg(surface-0).borderRWidth(1).borderR(border) }
  style brandSlot = { box: shrink(0).w(full).h(12).minH(12).padX(1).padY(1), paint: borderBWidth(1).borderB(border-subtle) }
  style brandMark = { box: w(10).h(10).minW(10).minH(10), paint: radius(md).bg(surface-1).fg(muted-foreground).borderWidth(1).border(border-subtle) }
  style itemsCol = { box: shrink(0).w(full).padY(2).padX(1) }
  style spacer = { box: grow(1).minH(0) }
  style footerCol = { box: shrink(0).w(full).padY(2).padX(1), paint: borderTWidth(1).borderT(border-subtle) }
</scope>

<nav label="Studio activities" style(railRoot)>
  <slot name="brand">
    <center hidden="true" style(brandSlot)>
      <center studio-brand-mark style(brandMark)>
        <icon name="ship" size="sm"></icon>
      </center>
    </center>
  </slot>
  <column studio-activity-items gap="1" align="center" style(itemsCol)>
    <slot></slot>
  </column>
  <column style(spacer)></column>
  <column studio-activity-footer align="center" gap="1" style(footerCol)>
    <slot name="footer"></slot>
  </column>
</nav>
