<!--
  Lesson 02 starter — the structure is right, the surface is bare.

  Style this usage card with concern-form Style: space({ pad }) / space({ pad-x,
  pad-y }), fill({ color }), ink({ color }), radius, border({ color, width }),
  elevation, type({ size, weight, tracking, leading }), and when({ hover, …
  focusWithin, current }) for interaction. Prefer a <scope> const when the
  same paint repeats (the three pills share one). Raw values (px, hex) do
  not compile — every setter argument is a token from a closed list.
-->
<column gap="4">
  <row gap="1">
    <!-- when({ current: … }) styles whichever pill carries
         aria-current="page". Style all three pills once; let the attribute
         pick the active one. -->
    <a href="#usage" aria-current="page">Usage</a>
    <a href="#billing">Billing</a>
    <a href="#limits">Limits</a>
  </row>

  <!-- Make this a card: fill({ color: surface-1 }), border, radius, space({ pad }),
       and a when({ hover: { paint: elevation(hover) } }) lift. -->
  <column gap="3">
    <row align="center" justify="between">
      <strong>Compute hours</strong>
      <small>renews in 12 days</small>
    </row>
    <row gap="2" align="end">
      <strong>412</strong>
      <small>of 500 hrs</small>
    </row>
    <p>Burst capacity unlocks automatically over 90% usage.</p>
    <!-- when({ focusWithin: style({ paint: border({ color: primary }) }) }) should light this
         row's border up when the input has focus. -->
    <row gap="2" align="center">
      <input placeholder="Set a usage alert (hours)" size="sm" />
      <button size="sm" variant="outline" tone="primary">Notify me</button>
    </row>
  </column>
</column>
