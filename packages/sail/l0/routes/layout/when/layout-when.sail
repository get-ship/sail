<!--
  Registered CE is src/layout-when/.
  Look: a painted box; primary by default, accent at tablet, muted at desktop
  Spelling: style box = { bg(primary), when: { tablet: bg(accent), desktop: bg(muted) } }
            Language keys + theme breakpoints. Colored-box demo, not an app remap.
  Owner: language keys, theme breakpoints
  Default: no breakpoint arms
  Trap: fixtures/when-unknown-key.bad.sail — Style error: style when key `phone` is not in the closed browser-state table
        Quoted below. Not compiled on this page.

        <column style({ bg(primary), when: { phone: bg(accent) } })>x</column>
-->
<scope>
  style box = { size({ w: 8, h: 8 }), bg(primary), when: { tablet: bg(accent), desktop: bg(muted) } }
</scope>
<column style(box)></column>
