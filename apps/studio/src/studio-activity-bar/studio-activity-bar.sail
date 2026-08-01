<!--
  <studio-activity-bar> — VS Code activity rail (fixed width, stable height chain).
  Width: one token w(12). Brand zone: h(8) full-rail — aligns with sidebar/tab headers.
  Slots: brand (top mark), default (icon links/buttons), footer (e.g. exit).
  Selected state: aria-current="page" on the active item; when({ current }) paints.
  Ban-clean: layout keywords + L1 only.
-->
<column
  data-studio-activity-bar
  role="navigation"
  aria-label="Studio activities"
  shrink="0"
  gap="1"
  align="center"
  sx={style.w(12).minH(0).h(full).bg(surface-0).borderRWidth(1).borderR(border).overflow(hidden)}
>
  <slot name="brand">
    <center
      shrink="0"
      aria-hidden="true"
      sx={style.w(full).h(8).borderBWidth(1).borderB(border-subtle)}
    >
      <center
        sx={style.w(6).h(6).radius(md).bg(primary).fg(primary-foreground).fontSize(xs).fontWeight(bold)}
      >
        S
      </center>
    </center>
  </slot>
  <slot></slot>
  <column grow sx={style.minH(0)}></column>
  <slot name="footer"></slot>
</column>
