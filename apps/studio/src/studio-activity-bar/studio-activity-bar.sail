<!--
  <studio-activity-bar> — VS Code activity rail.
  Outer width 12; items 10×10 with pad-x 1. overflow visible so right tooltips are not clipped.
  Slots: brand, default (items), footer.
-->
<scope>
  style railRoot = { display(flex).dir(column).shrink(0).size({ w: 12, h: full, min-w: 12, min-h: 0, max-w: 12 }).overflow({ all: visible }).bg(surface-0).border({ edge: r, color: border, width: 1 }) }
  style brandSlot = { shrink(0).size({ w: full, h: 12, min-h: 12 }).pad({ x: 1, y: 1 }).border({ edge: b, color: border-subtle, width: 1 }) }
  style brandMark = { size({ w: 10, h: 10, min-w: 10, min-h: 10 }).radius(md).bg(surface-1).fg(muted-foreground).border({ color: border-subtle, width: 1 }) }
  style itemsCol = { shrink(0).size({ w: full }).pad({ x: 1, y: 2 }) }
  style spacer = { grow(1).size({ min-h: 0 }) }
  style footerCol = { shrink(0).size({ w: full }).pad({ x: 1, y: 2 }).border({ edge: t, color: border-subtle, width: 1 }) }
</scope>

<nav label="Studio activities" style(railRoot)>
  <if {false}>
    <icon name="file"></icon>
    <icon name="compass"></icon>
    <icon name="image"></icon>
    <icon name="home"></icon>
    <icon name="circle"></icon>
    <icon name="star"></icon>
    <icon name="user"></icon>
    <icon name="bell"></icon>
    <icon name="mail"></icon>
    <icon name="check"></icon>
    <icon name="close"></icon>
    <icon name="chevron-right"></icon>
    <icon name="trash"></icon>
    <icon name="edit"></icon>
    <icon name="download"></icon>
    <icon name="file-code"></icon>
  </if>
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
