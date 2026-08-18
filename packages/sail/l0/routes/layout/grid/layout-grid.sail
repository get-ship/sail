<!--
  Registered CE is src/layout-grid/.
  Look: four 4-step boxes in a 2-column grid, 8px gap
  Spelling: <grid cols="2" gap="2"> + four painted children
            (bg(primary) / bg(accent) / bg(muted) / bg(primary)). 8px is space-2.
  Owner: language (grid host)
  Default: min=md auto-fill
  Trap: fixtures/grid-cols-off-enum.bad.sail — <grid cols="99"> — `99` is not valid. allowed: 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12
        Quoted below. Not compiled on this page.

        <grid cols="99">x</grid>
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style hug = { size(hug) }
  style a = { size({ w: 16, h: 8 }), bg(primary) }
  style b = { size({ w: 16, h: 8 }), bg(accent) }
  style c = { size({ w: 16, h: 8 }), bg(muted) }
  style d = { size({ w: 16, h: 8 }), bg(primary) }
  style cell = { size({ h: 8 }), bg(primary) }
  style cellA = { size({ h: 8 }), bg(accent) }
  style cellM = { size({ h: 8 }), bg(muted) }
</scope>
<column gap="6">
  <column gap="2">
    <p style(look)>2-col hug grid, four 16×8 bars, 8px gap</p>
    <p style(spell)>&lt;grid cols="2" gap="2"&gt; size(hug)</p>
    <grid cols="2" gap="2" style(hug)>
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
      <column style(d)></column>
    </grid>
  </column>
  <column gap="2">
    <p style(look)>cols=1</p>
    <p style(spell)>&lt;grid cols="1" gap="2"&gt;</p>
    <grid cols="1" gap="2" style(hug)>
      <column style(a)></column>
      <column style(b)></column>
    </grid>
  </column>
  <column gap="2">
    <p style(look)>cols=3</p>
    <p style(spell)>&lt;grid cols="3" gap="2"&gt;</p>
    <grid cols="3" gap="2" style(hug)>
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </grid>
  </column>
  <column gap="2">
    <p style(look)>cols=4</p>
    <p style(spell)>&lt;grid cols="4" gap="2"&gt;</p>
    <grid cols="4" gap="2" style(hug)>
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
      <column style(d)></column>
    </grid>
  </column>
  <column gap="2">
    <p style(look)>min=xs</p>
    <p style(spell)>&lt;grid min="xs" gap="2"&gt;</p>
    <grid min="xs" gap="2">
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
    </grid>
  </column>
  <column gap="2">
    <p style(look)>min=sm</p>
    <p style(spell)>&lt;grid min="sm" gap="2"&gt;</p>
    <grid min="sm" gap="2">
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
    </grid>
  </column>
  <column gap="2">
    <p style(look)>min=md</p>
    <p style(spell)>&lt;grid min="md" gap="2"&gt;</p>
    <grid min="md" gap="2">
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
    </grid>
  </column>
  <column gap="2">
    <p style(look)>min=lg</p>
    <p style(spell)>&lt;grid min="lg" gap="2"&gt;</p>
    <grid min="lg" gap="2">
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
    </grid>
  </column>
  <column gap="2">
    <p style(look)>flow=auto</p>
    <p style(spell)>&lt;grid cols="3" flow="auto" gap="2"&gt;</p>
    <grid cols="3" flow="auto" gap="2" style(hug)>
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
      <column style(d)></column>
    </grid>
  </column>
  <column gap="2">
    <p style(look)>flow=dense</p>
    <p style(spell)>&lt;grid cols="3" flow="dense" gap="2"&gt;</p>
    <grid cols="3" flow="dense" gap="2" style(hug)>
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
      <column style(d)></column>
    </grid>
  </column>
</column>
