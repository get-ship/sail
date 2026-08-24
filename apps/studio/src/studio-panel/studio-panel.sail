<!--
  <studio-panel> — bordered workbench panel (header + body + footer).
  Header height 8 matches shell header rhythm. Props: title. Slots: header, actions, default, footer.
-->
<scope>
  props: {
    title = ''
  }
  style panelRoot = { size(grow).overflow(hidden).border(subtle).radius(md).bg(surface) }
  // no preset: size lock + edge-only border
  style panelHeader = { size({ h: 8, min-h: 8 }).pad(3).border(subtle-b) }
  style eyebrow = { fg(muted).type(label) }
  style actionsRow = { size(hug) }
  style panelBody = { size(grow).overflow(hidden) }
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
