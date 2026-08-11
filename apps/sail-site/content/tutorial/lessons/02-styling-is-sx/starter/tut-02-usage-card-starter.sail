<!--
  Lesson 02 starter — the structure is right, the surface is bare.

  Style this usage card with the Style product alone: style({ box: pad }) / padX /
  padY, style({ paint: bg }) / fg, style({ paint: radius }), style({ paint: borderWidth }) + border, style({ paint: shadow }),
  type (fontSize / fontWeight / tracking / leading), and when({ hover, …
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

  <!-- Make this a card: surface bg, border, radius, padding, and a
       when({ hover: style({ paint: shadow(hover) }) }) lift. -->
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
    <!-- when({ focusWithin: style({ paint: border(primary) }) }) should light this
         row's border up when the input has focus. -->
    <row gap="2" align="center">
      <input placeholder="Set a usage alert (hours)" size="sm" />
      <button size="sm" variant="outline" tone="primary">Notify me</button>
    </row>
  </column>
</column>
