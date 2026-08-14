<!--
  <studio-panel> — bordered workbench panel (header + body + footer).
  Header height 8 matches shell header rhythm. Props: title. Slots: header, actions, default, footer.
-->
<scope>
  props: {
    title = ''
  }
  style panelRoot = { grow(1).size({ h: full, min-w: 0, min-h: 0 }).overflow({ all: hidden }).border({ color: border-subtle, width: 1 }).radius(md).bg(surface-1) }
  style panelHeader = { shrink(0).size({ h: 8, min-h: 8 }).pad({ x: 3 }).border({ edge: b, color: border-subtle, width: 1 }) }
  style eyebrow = { fg(muted-foreground).type({ size: xs, weight: semibold, tracking: wide, case: uppercase }) }
  style actionsRow = { shrink(0) }
  style panelBody = { grow(1).size({min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }) }
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
