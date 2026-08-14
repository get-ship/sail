<!--
  Lesson 02 solved — the whole surface is the Style product.

  Spacing, color, radius, border, elevation, and type are tokens via a
  Style chain; interaction states are when({ hover, focusWithin,
  current, … }). Nothing here is a raw value — a px or a hex would not
  compile. Named scope Style (style name = { … }) is reused on tags as
  style(name).
-->
<scope>
  style pill = { pad({ x: 3, y: 1 }).radius(full).fg(muted-foreground).type({ size: sm, weight: semibold }), when: { hover: fg(foreground).bg(surface-2), current: bg(primary).fg(primary-foreground) } }
  style card = { pad(5).bg(surface-1).border({ color: border, width: 1 }).radius(lg).elevation(sm), when: { hover: elevation(hover) } }
  style label = { fg(muted-foreground).type({ size: sm, tracking: wide }) }
  style muted = { fg(muted-foreground) }
  style value = { type({ size: 3xl, tracking: tight }) }
  style of = { pad({ b: 1 }).fg(muted-foreground) }
  style body = { fg(muted-foreground).type({ size: sm, leading: relaxed }) }
  style alertRow = { pad(2).radius(md).border({ color: border-subtle, width: 1 }), when: { focusWithin: border({ color: primary }) } }
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
