<!--
  Look: the inspector line quotes .s-bg-surface and .s-pad-x4-y16.
  Spelling: bg(surface) · pad({ x: 4, y: 16 })
  Owner: language (BUCKETS §2.6 class names)
  Default: no class
  Trap: none this slice — vocab_class.rs pins the names.
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style specimen = { size({ w: 16, h: 8 }), pad({ x: 4, y: 16 }), bg(surface), fg(muted), type(mono), radius(sm) }
</scope>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>class names read as vocabulary — .s-bg-surface · .s-pad-x4-y16</p>
    <p slot="spell" style(spell)>bg(surface) · pad x:4 y:16</p>
    <column data-inspect-specimen style(specimen)>surface</column>
  </l0-case>
</column>
