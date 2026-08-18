<scope>
  style shell = { pad(page) }
  style kicker = { type(label).fg(muted) }
  style pane = { bg(muted) }
</scope>

<column gap="4" style(shell)>
  <row gap="3" align="center" justify="between">
    <theme-picker></theme-picker>
    <small style(kicker)>viewport — resize the window</small>
  </row>
  <row gap="4">
    <column gap="2" style(pane)>
      <small style(kicker)>live</small>
      <slot></slot>
    </column>
    <column gap="2" style(pane)>
      <small style(kicker)>takumi</small>
    </column>
  </row>
</column>
