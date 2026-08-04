<!--
  <studio-activity-bar> — VS Code activity rail.
  Fixed outer width w(12); inner items are w/h 10 with padX(1) so hit targets
  center consistently (no mixed sizes). Brand mark matches item box geometry.
  overflow visible so right tooltips are not clipped.
  Slots: brand, default (items), footer (exit — same item style as activities).
  Ban-clean: layout keywords + L1 only.
-->
<nav label="Studio activities" shrink="0" sx={style.display(flex).flexDir(column).w(12).minW(12).maxW(12).minH(0).h(full).bg(surface-0).borderRWidth(1).borderR(border).overflow(visible)}>
  <slot name="brand">
    <center
      shrink="0"
      hidden="true"
      sx={style.w(full).h(12).minH(12).padX(1).padY(1).borderBWidth(1).borderB(border-subtle)}
    >
      <center
        studio-brand-mark
        sx={style.w(10).h(10).minW(10).minH(10).radius(md).bg(surface-1).fg(muted-foreground).borderWidth(1).border(border-subtle)}
      >
        <icon name="ship" size="sm"></icon>
      </center>
    </center>
  </slot>
  <column
    studio-activity-items
    gap="1"
    align="center"
    shrink="0"
    sx={style.w(full).padY(2).padX(1)}
  >
    <slot></slot>
  </column>
  <column grow sx={style.minH(0)}></column>
  <column
    studio-activity-footer
    shrink="0"
    align="center"
    gap="1"
    sx={style.w(full).padY(2).padX(1).borderTWidth(1).borderT(border-subtle)}
  >
    <slot name="footer"></slot>
  </column>
</nav>
