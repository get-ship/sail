<!--
  Lesson 02 starter — the structure is right, the surface is bare.

  Style this usage card with named presets: pad(5) / pad(2), bg(surface),
  fg(muted), radius(lg), border(subtle), elevation(sm), type(label),
  type(body-sm), type(display-sm), and when: { hover, focusWithin,
  current } for interaction. Prefer a <scope> const when the same paint
  repeats (the three pills share one). Raw values (px, hex) do not
  compile — every setter argument is a token from a closed list.

  Slot recipes (type&#40;&#123; size: sm &#125;&#41;, pad&#40;&#123; x: 3 &#125;&#41;,
  border&#40;&#123; color, width &#125;&#41;) are L1-only. Spend the preset.
-->
<column gap="4">
  <row gap="1">
    <!-- when: { current: … } styles whichever pill carries
         aria-current="page". Style all three pills once; let the attribute
         pick the active one. -->
    <a href="#usage" aria-current="page">Usage</a>
    <a href="#billing">Billing</a>
    <a href="#limits">Limits</a>
  </row>

  <!-- Make this a card: bg(surface), border(subtle), radius, pad(5),
       and a when: { hover: elevation(hover) } lift. -->
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
    <!-- when: { focusWithin: border(strong) } should light this
         row's border up when the input has focus. -->
    <row gap="2" align="center">
      <input placeholder="Set a usage alert (hours)" size="sm" />
      <button size="sm" variant="outline" tone="primary">Notify me</button>
    </row>
  </column>
</column>
