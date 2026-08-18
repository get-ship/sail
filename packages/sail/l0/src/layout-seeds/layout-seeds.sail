<!--
  Look: named seed cases as colored boxes. No off-scale pixel looks.
  Spelling: one cell per seed, hosts + participation + free size slots.
  Owner: language (structure) + free geometry
  Default: —
  Trap: none on this page (traps live on the verb pages)
  Skipped: aspect-vs-grow — size(grow) and size({ aspect: square }) clash
           on size@base. No legal spelling without changing the spec.
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style box4 = { size({ w: 8, h: 8 }), bg(primary) }
  style box4a = { size({ w: 8, h: 8 }), bg(accent) }
  style box4m = { size({ w: 8, h: 8 }), bg(muted) }
  style leftover = { size(grow), pad(3), bg(accent) }
  style rail = { size({ w: 16 }), pad(3), bg(primary) }
  style fluid = { size(grow), pad(3), bg(accent) }
  style header = { size({ h: 8 }), pad(3), bg(primary) }
  style footer = { size({ h: 8 }), pad(3), bg(muted) }
  style nav = { size({ w: 16 }), pad(3), bg(primary) }
  style main = { size(grow), pad(3), bg(accent) }
  style aside = { size({ w: 16 }), pad(3), bg(muted) }
  style pctParent = { size({ w: 50% }), pad(3), bg(muted) }
  style pctChild = { size({ w: 50% }), pad(3), bg(primary) }
  style spanA = { pad(3), bg(primary) }
  style spanB = { pad(3), bg(accent) }
  style spanC = { pad(3), bg(muted) }
  style overlapA = { size({ w: 16, h: 16 }), bg(primary) }
  style overlapB = { size({ w: 16, h: 16 }), bg(accent) }
  style fill = { size({ w: 100% }) }
</scope>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>grow-without-bound — grow child, host has no width</p>
    <p slot="spell" style(spell)>size(grow) in &lt;row&gt;</p>
    <row>
      <column style(leftover)>grow</column>
    </row>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>circular-percentage — child 50% of a 50% parent</p>
    <p slot="spell" style(spell)>size w:50% nested</p>
    <column style(pctParent)>
      <column style(pctChild)>pct</column>
    </column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>RTL wrap — dir=rtl, 8px gap</p>
    <p slot="spell" style(spell)>&lt;row dir="rtl" gap="2"&gt;</p>
    <row dir="rtl" gap="2">
      <column style(box4)></column>
      <column style(box4a)></column>
      <column style(box4m)></column>
    </row>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>sibling-overlap opt-in — stack overlays</p>
    <p slot="spell" style(spell)>&lt;stack&gt;</p>
    <stack>
      <column style(overlapA)></column>
      <column style(overlapB)></column>
    </stack>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>holy-grail — header / nav+main+aside / footer</p>
    <p slot="spell" style(spell)>column of rows · nav/aside w:16 · main grow</p>
    <column style(fill)>
      <row style(header)>header</row>
      <row nowrap>
        <column style(nav)>nav</column>
        <column style(main)>main</column>
        <column style(aside)>aside</column>
      </row>
      <row style(footer)>footer</row>
    </column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>sidebar-fixed-fluid — 16-step rail + grow</p>
    <p slot="spell" style(spell)>size w:16 + size(grow)</p>
    <row nowrap style(fill)>
      <column style(rail)>rail</column>
      <column style(fluid)>fluid</column>
    </row>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>bento spanning — col-span / row-span on 4 tracks</p>
    <p slot="spell" style(spell)>&lt;grid cols="4"&gt; col-span=2 row-span=2</p>
    <grid cols="4" gap="2">
      <column col-span="2" row-span="2" style(spanA)>a</column>
      <column style(spanB)>b</column>
      <column style(spanC)>c</column>
      <column col-span="2" style(spanB)>d</column>
    </grid>
  </l0-case>
</column>
