<!--
  Lesson 02 solved — the whole surface is the Style product.

  Spacing, color, radius, border, shadow, and type are tokens via Style
  object channels; interaction states are when({ hover, focusWithin,
  current, … }). Nothing here is a raw value — a px or a hex would not
  compile. Named scope Style (style name = { … }) is reused on tags as
  style(name).
-->
<scope>
  style pill = { box: padX(3).padY(1), paint: radius(full).fg(muted-foreground), text: fontSize(sm).fontWeight(semibold), when: { hover: { paint: fg(foreground).bg(surface-2) }, current: { paint: bg(primary).fg(primary-foreground) } } }
  style card = { box: pad(5), paint: bg(surface-1).borderWidth(1).border(border).radius(lg).shadow(sm), when: { hover: { paint: shadow(hover) } } }
  style label = { paint: fg(muted-foreground), text: fontSize(sm).tracking(wide) }
  style muted = { paint: fg(muted-foreground) }
  style value = { text: fontSize(3xl).tracking(tight) }
  style of = { box: padB(1), paint: fg(muted-foreground) }
  style body = { paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed) }
  style alertRow = { box: pad(2), paint: radius(md).borderWidth(1).border(border-subtle), when: { focusWithin: { paint: border(primary) } } }
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
