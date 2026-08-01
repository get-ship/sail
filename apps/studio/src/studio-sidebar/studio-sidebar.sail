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

<column
  data-studio-sidebar
  shrink="0"
  gap="0"
  sx={style.w(64).minW(64).minH(0).h(full).borderRWidth(1).borderR(border).bg(surface-1).overflow(hidden)}
>
  <slot name="header">
    <row
      data-studio-sidebar-header
      align="center"
      shrink="0"
      sx={style.h(8).minH(8).padX(3).borderBWidth(1).borderB(border-subtle)}
    >
      <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
        {title()}
      </p>
    </row>
  </slot>
  <slot name="toolbar"></slot>
  <column
    data-studio-sidebar-body
    grow
    gap="0"
    min-w="0"
    sx={style.minH(0).h(full).basis(0).overflow(hidden)}
  >
    <slot></slot>
  </column>
</column>
