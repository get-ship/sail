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
  style box4 = { size({ w: 4, h: 4 }), bg(primary) }
  style box4a = { size({ w: 4, h: 4 }), bg(accent) }
  style box4m = { size({ w: 4, h: 4 }), bg(muted) }
  style leftover = { size(grow), bg(accent) }
  style rail = { size({ w: 8 }), bg(primary) }
  style fluid = { size(grow), bg(accent) }
  style header = { size({ h: 4 }), bg(primary) }
  style footer = { size({ h: 4 }), bg(muted) }
  style nav = { size({ w: 8 }), bg(primary) }
  style main = { size(grow), bg(accent) }
  style aside = { size({ w: 8 }), bg(muted) }
  style pctParent = { size({ w: 50% }), bg(muted) }
  style pctChild = { size({ w: 50% }), bg(primary) }
  style spanA = { bg(primary) }
  style spanB = { bg(accent) }
  style spanC = { bg(muted) }
  style overlapA = { size({ w: 8, h: 8 }), bg(primary) }
  style overlapB = { size({ w: 8, h: 8 }), bg(accent) }
</scope>
<column gap="4">
  <!-- grow-without-bound: grow child, host has no explicit width -->
  <row>
    <column style(leftover)>grow</column>
  </row>
  <!-- circular-percentage -->
  <column style(pctParent)>
    <column style(pctChild)>pct</column>
  </column>
  <!-- RTL wrap: dir is HTML passthrough; 8px is space-2 -->
  <row dir="rtl" gap="2">
    <column style(box4)></column>
    <column style(box4a)></column>
    <column style(box4m)></column>
  </row>
  <!-- sibling-overlap opt-in: stack is the overlay host -->
  <stack>
    <column style(overlapA)></column>
    <column style(overlapB)></column>
  </stack>
  <!-- holy-grail -->
  <column>
    <row style(header)>header</row>
    <row nowrap>
      <column style(nav)>nav</column>
      <column style(main)>main</column>
      <column style(aside)>aside</column>
    </row>
    <row style(footer)>footer</row>
  </column>
  <!-- sidebar-fixed-fluid -->
  <row nowrap>
    <column style(rail)>rail</column>
    <column style(fluid)>fluid</column>
  </row>
  <!-- bento spanning -->
  <grid cols="4" gap="2">
    <column col-span="2" row-span="2" style(spanA)>a</column>
    <column style(spanB)>b</column>
    <column style(spanC)>c</column>
    <column col-span="2" style(spanB)>d</column>
  </grid>
</column>
