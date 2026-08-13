<!--
  <studio-activity-bar> — VS Code activity rail.
  Outer width 12; items 10×10 with pad-x 1. overflow visible so right tooltips are not clipped.
  Slots: brand, default (items), footer.
-->
<scope>
  style railRoot = { box: flow({ display: flex, dir: column, shrink: 0 }).size({ w: 12, h: full, min-w: 12, min-h: 0, max-w: 12 }).overflow({ all: visible }), paint: fill({ color: surface-0 }).border({ edge: r, color: border, width: 1 }) }
  style brandSlot = { box: flow({ shrink: 0 }).size({ w: full, h: 12, min-h: 12 }).space({ pad-x: 1, pad-y: 1 }), paint: border({ edge: b, color: border-subtle, width: 1 }) }
  style brandMark = { box: size({ w: 10, h: 10, min-w: 10, min-h: 10 }), paint: radius(md).fill({ color: surface-1 }).ink({ color: muted-foreground }).border({ color: border-subtle, width: 1 }) }
  style itemsCol = { box: flow({ shrink: 0 }).size({ w: full }).space({ pad-x: 1, pad-y: 2 }) }
  style spacer = { box: flow({ grow: 1 }).size({ min-h: 0 }) }
  style footerCol = { box: flow({ shrink: 0 }).size({ w: full }).space({ pad-x: 1, pad-y: 2 }), paint: border({ edge: t, color: border-subtle, width: 1 }) }
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
