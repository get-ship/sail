<!--
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
  <l0-case>
    <p slot="look" style(look)>2-col hug grid, four 16×8 bars, 8px gap</p>
    <p slot="spell" style(spell)>&lt;grid cols="2" gap="2"&gt; size(hug)</p>
    <grid cols="2" gap="2" style(hug)>
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
      <column style(d)></column>
    </grid>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>cols=1</p>
    <p slot="spell" style(spell)>&lt;grid cols="1" gap="2"&gt;</p>
    <grid cols="1" gap="2" style(hug)>
      <column style(a)></column>
      <column style(b)></column>
    </grid>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>cols=3</p>
    <p slot="spell" style(spell)>&lt;grid cols="3" gap="2"&gt;</p>
    <grid cols="3" gap="2" style(hug)>
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </grid>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>cols=4</p>
    <p slot="spell" style(spell)>&lt;grid cols="4" gap="2"&gt;</p>
    <grid cols="4" gap="2" style(hug)>
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
      <column style(d)></column>
    </grid>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>cols=5</p>
    <p slot="spell" style(spell)>&lt;grid cols="5" gap="2"&gt;</p>
    <grid cols="5" gap="2" style(hug)>
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
      <column style(cellA)></column>
    </grid>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>cols=6</p>
    <p slot="spell" style(spell)>&lt;grid cols="6" gap="2"&gt;</p>
    <grid cols="6" gap="2" style(hug)>
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
    </grid>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>cols=7</p>
    <p slot="spell" style(spell)>&lt;grid cols="7" gap="2"&gt;</p>
    <grid cols="7" gap="2" style(hug)>
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
    </grid>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>cols=8</p>
    <p slot="spell" style(spell)>&lt;grid cols="8" gap="2"&gt;</p>
    <grid cols="8" gap="2" style(hug)>
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
      <column style(cellA)></column>
    </grid>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>cols=9</p>
    <p slot="spell" style(spell)>&lt;grid cols="9" gap="2"&gt;</p>
    <grid cols="9" gap="2" style(hug)>
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
    </grid>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>cols=10</p>
    <p slot="spell" style(spell)>&lt;grid cols="10" gap="2"&gt;</p>
    <grid cols="10" gap="2" style(hug)>
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
    </grid>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>cols=11</p>
    <p slot="spell" style(spell)>&lt;grid cols="11" gap="2"&gt;</p>
    <grid cols="11" gap="2" style(hug)>
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
      <column style(cellA)></column>
    </grid>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>cols=12</p>
    <p slot="spell" style(spell)>&lt;grid cols="12" gap="2"&gt;</p>
    <grid cols="12" gap="2" style(hug)>
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
    </grid>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>min=xs</p>
    <p slot="spell" style(spell)>&lt;grid min="xs" gap="2"&gt;</p>
    <grid min="xs" gap="2">
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
    </grid>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>min=sm</p>
    <p slot="spell" style(spell)>&lt;grid min="sm" gap="2"&gt;</p>
    <grid min="sm" gap="2">
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
    </grid>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>min=md</p>
    <p slot="spell" style(spell)>&lt;grid min="md" gap="2"&gt;</p>
    <grid min="md" gap="2">
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
    </grid>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>min=lg</p>
    <p slot="spell" style(spell)>&lt;grid min="lg" gap="2"&gt;</p>
    <grid min="lg" gap="2">
      <column style(cell)></column>
      <column style(cellA)></column>
      <column style(cellM)></column>
      <column style(cell)></column>
    </grid>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>flow=auto</p>
    <p slot="spell" style(spell)>&lt;grid cols="3" flow="auto" gap="2"&gt;</p>
    <grid cols="3" flow="auto" gap="2" style(hug)>
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
      <column style(d)></column>
    </grid>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>flow=dense</p>
    <p slot="spell" style(spell)>&lt;grid cols="3" flow="dense" gap="2"&gt;</p>
    <grid cols="3" flow="dense" gap="2" style(hug)>
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
      <column style(d)></column>
    </grid>
  </l0-case>
</column>
