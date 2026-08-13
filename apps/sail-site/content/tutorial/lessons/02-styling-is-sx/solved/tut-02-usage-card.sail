<!--
  Lesson 02 solved — the whole surface is the Style product.

  Spacing, color, radius, border, shadow, and type are tokens via Style
  object channels; interaction states are when({ hover, focusWithin,
  current, … }). Nothing here is a raw value — a px or a hex would not
  compile. Named scope Style (style name = { … }) is reused on tags as
  style(name).
-->
<scope>
  style pill = { box: space({ pad-x: 3, pad-y: 1 }), paint: radius(full).ink({ color: muted-foreground }), text: type({ size: sm, weight: semibold }), when: { hover: { paint: ink({ color: foreground }).fill({ color: surface-2 }) }, current: { paint: fill({ color: primary }).ink({ color: primary-foreground }) } } }
  style card = { box: space({ pad: 5 }), paint: fill({ color: surface-1 }).border({ color: border, width: 1 }).radius(lg).elevation(sm), when: { hover: { paint: elevation(hover) } } }
  style label = { paint: ink({ color: muted-foreground }), text: type({ size: sm, tracking: wide }) }
  style muted = { paint: ink({ color: muted-foreground }) }
  style value = { text: type({ size: 3xl, tracking: tight }) }
  style of = { box: space({ pad-b: 1 }), paint: ink({ color: muted-foreground }) }
  style body = { paint: ink({ color: muted-foreground }), text: type({ size: sm, leading: relaxed }) }
  style alertRow = { box: space({ pad: 2 }), paint: radius(md).border({ color: border-subtle, width: 1 }), when: { focusWithin: { paint: border({ color: primary }) } } }
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
