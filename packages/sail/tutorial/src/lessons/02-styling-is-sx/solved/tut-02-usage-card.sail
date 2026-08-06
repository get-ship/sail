<!--
  Lesson 02 solved — the whole surface is Style on sx.

  Spacing, color, radius, border, shadow, and type are tokens via style.*
  chains; interaction states are when({ hover, focusWithin, current, … }).
  Nothing here is a raw value — a px or a hex would not compile.
-->
<scope>
  const pill = style.padX(3).padY(1).radius(full).fontSize(sm).fontWeight(semibold).fg(muted-foreground).when({
    hover: style.fg(foreground).bg(surface-2),
    current: style.bg(primary).fg(primary-foreground),
  })
  const card = style.pad(5).bg(surface-1).borderWidth(1).border(border).radius(lg).shadow(sm).when({
    hover: style.shadow(hover),
  })
  const label = style.fontSize(sm).tracking(wide).fg(muted-foreground)
  const muted = style.fg(muted-foreground)
  const value = style.fontSize(3xl).tracking(tight)
  const of = style.fg(muted-foreground).padB(1)
  const body = style.fontSize(sm).fg(muted-foreground).leading(relaxed)
  const alertRow = style.pad(2).radius(md).borderWidth(1).border(border-subtle).when({
    focusWithin: style.border(primary),
  })
</scope>

<column gap="4">
  <row gap="1">
    <a href="#usage" aria-current="page">
      style(pill)
Usage</a>
    <a href="#billing">
      style(pill)
Billing</a>
    <a href="#limits">
      style(pill)
Limits</a>
  </row>

  <column gap="3">
    style(card)

    <row align="center" justify="between">
      <strong>
        style(label)
Compute hours</strong>
      <small>
        style(muted)
renews in 12 days</small>
    </row>
    <row gap="2" align="end">
      <strong>
        style(value)
412</strong>
      <small>
        style(of)
of 500 hrs</small>
    </row>
    <p>
      style(body)
Burst capacity unlocks automatically over 90% usage.</p>
    <row gap="2" align="center">
      style(alertRow)

      <input placeholder="Set a usage alert (hours)" size="sm" />
      <button size="sm" variant="outline" tone="primary">Notify me</button>
    </row>
  </column>
</column>
