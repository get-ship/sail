<!--
  <studio-activity-bar> — VS Code activity rail.
  Outer width 12; items 10×10 with pad-x 1. overflow visible so right tooltips are not clipped.
  Slots: brand, default (items), footer.
-->
<scope>
  // no preset: size lock + edge-only border
  style railRoot = { display(flex).dir(column).size({ w: 12, h: 100%, min-w: 12, min-h: 0, max-w: 12 }).overflow(visible).bg(app).border(default-r) }
  // no preset: size lock + edge-only border
  style brandSlot = { size({ w: 100%, h: 12, min-h: 12 }).pad(1).border(subtle-b) }
  // no preset: size lock
  style brandMark = { size({ w: 10, h: 10, min-w: 10, min-h: 10 }).radius(md).bg(surface).fg(muted).border(subtle) }
  style itemsCol = { size(max).pad(1) }
  style spacer = { size(max) }
  // no preset: edge-only border
  style footerCol = { size(max).pad(1).border(subtle-t) }
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
