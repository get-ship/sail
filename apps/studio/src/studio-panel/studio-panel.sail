<!--
  <studio-panel> — bordered workbench panel (header + body + footer).
  Header h(8) matches shell header rhythm (sidebar, brand, edit tabs).
  Props: title. Slots: header (overrides title row), actions, default, footer.
  Ban-clean: layout keywords + L1 only (bordered column, not a parallel system).
-->
<scope>
  props: {
    title = ''
  }
</scope>

<column studio-panel gap="0">
  style({  box: grow(1).minW(0).minH(0).minW(0).h(full).overflow(hidden), paint: borderWidth(1).border(border-subtle).radius(md).bg(surface-1)  })

  <slot name="header">
    <row studio-panel-header align="center" gap="2" nowrap justify="between">
      style({  box: shrink(0).h(8).minH(8).padX(3), paint: borderBWidth(1).borderB(border-subtle)  })

      <p>
        style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide) })

        {title()}
      </p>
      <row align="center" gap="1" nowrap>
        style({ box: shrink(0) })

        <slot name="actions"></slot>
      </row>
    </row>
  </slot>
  <column studio-panel-body gap="0">
    style({  box: grow(1).minW(0).minH(0).basis(0).overflow(hidden)  })

    <slot></slot>
  </column>
  <slot name="footer"></slot>
</column>
