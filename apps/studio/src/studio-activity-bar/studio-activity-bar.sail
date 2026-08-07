<!--
  <studio-activity-bar> — VS Code activity rail.
  Fixed outer width w(12); inner items are w/h 10 with padX(1) so hit targets
  center consistently (no mixed sizes). Brand mark matches item box geometry.
  overflow visible so right tooltips are not clipped.
  Slots: brand, default (items), footer (exit — same item style as activities).
  Ban-clean: layout keywords + L1 only.
-->
<nav label="Studio activities" style({  box: shrink(0).display(flex).flexDir(column).w(12).minW(12).maxW(12).minH(0).h(full).overflow(visible), paint: bg(surface-0).borderRWidth(1).borderR(border)  })>
  <slot name="brand">
    <center hidden="true" style({  box: shrink(0).w(full).h(12).minH(12).padX(1).padY(1), paint: borderBWidth(1).borderB(border-subtle)  })>
      <center
        studio-brand-mark style({ box: w(10).h(10).minW(10).minH(10), paint: radius(md).bg(surface-1).fg(muted-foreground).borderWidth(1).border(border-subtle) })>
        <icon name="ship" size="sm"></icon>
      </center>
    </center>
  </slot>
  <column studio-activity-items gap="1" align="center" style({  box: shrink(0).w(full).padY(2).padX(1)  })>
    <slot></slot>
  </column>
  <column style({  box: grow(1).minH(0)  })></column>
  <column studio-activity-footer align="center" gap="1" style({  box: shrink(0).w(full).padY(2).padX(1), paint: borderTWidth(1).borderT(border-subtle)  })>
    <slot name="footer"></slot>
  </column>
</nav>
