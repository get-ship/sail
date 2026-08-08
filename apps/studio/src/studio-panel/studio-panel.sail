<!--
  <studio-panel> — bordered workbench panel (header + body + footer).
  Header h(8) matches shell header rhythm. Props: title. Slots: header, actions, default, footer.
-->
<scope>
  props: {
    title = ''
  }
  style panelRoot = { box: grow(1).minW(0).minH(0).h(full).overflow(hidden), paint: borderWidth(1).border(border-subtle).radius(md).bg(surface-1) }
  style panelHeader = { box: shrink(0).h(8).minH(8).padX(3), paint: borderBWidth(1).borderB(border-subtle) }
  style eyebrow = { paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide) }
  style actionsRow = { box: shrink(0) }
  style panelBody = { box: grow(1).minW(0).minH(0).basis(0).overflow(hidden) }
</scope>

<column studio-panel gap="0" style(panelRoot)>
  <slot name="header">
    <row studio-panel-header align="center" gap="2" nowrap justify="between" style(panelHeader)>
      <p style(eyebrow)>
        {title()}
      </p>
      <row align="center" gap="1" nowrap style(actionsRow)>
        <slot name="actions"></slot>
      </row>
    </row>
  </slot>
  <column studio-panel-body gap="0" style(panelBody)>
    <slot></slot>
  </column>
  <slot name="footer"></slot>
</column>
