<!--
  Lesson 02 solved — the whole surface is the Style product.

  Spacing, color, radius, border, elevation, and type are theme presets;
  interaction states are when: { hover, focusWithin, current, … }.
  Nothing here is a raw value — a px or a hex would not compile. Named
  scope Style (style name = { … }) is reused on tags as style(name).
-->
<scope>
  style pill = { pad(2).radius(full).fg(muted).type(label), when: { hover: fg(foreground).bg(muted), current: bg(primary).fg(primary-foreground) } }
  style card = { pad(5).bg(surface).border(subtle).radius(lg).elevation(sm), when: { hover: elevation(hover) } }
  style label = { fg(muted).type(label) }
  style muted = { fg(muted) }
  style value = { type(display-sm) }
  style of = { pad(1).fg(muted) }
  style body = { fg(muted).type(body-sm) }
  style alertRow = { pad(2).radius(md).border(subtle), when: { focusWithin: border(strong) } }
</scope>

<column gap="4">
  <row gap="1">
    <a href="#usage" aria-current="page" style(pill)>Usage</a>
    <a href="#billing" style(pill)>Billing</a>
    <a href="#limits" style(pill)>Limits</a>
  </row>

  <column gap="3" style(card)>
    <row align="center" justify="between">
      <strong style(label)>Compute hours</strong>
      <small style(muted)>renews in 12 days</small>
    </row>
    <row gap="2" align="end">
      <strong style(value)>412</strong>
      <small style(of)>of 500 hrs</small>
    </row>
    <p style(body)>Burst capacity unlocks automatically over 90% usage.</p>
    <row gap="2" align="center" style(alertRow)>
      <input placeholder="Set a usage alert (hours)" size="sm" />
      <button size="sm" variant="outline" tone="primary">Notify me</button>
    </row>
  </column>
</column>
