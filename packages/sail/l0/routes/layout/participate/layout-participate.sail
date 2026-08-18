<!--
  Registered CE is src/layout-participate/.
  Look: grow / hug / max per host.
        row: hug child + grow leftover; hug child + max stretch.
        column: hug child + grow leftover (host is 16 tall).
        grid: hug cell + grow cell.
        center: hug child.
        stack: hug layer over a muted box.
  Spelling: size(hug) / size(grow) / size(max) on children of
            <row> <column> <grid> <center> <stack>. Paint with named bg only.
  Owner: language (participation)
  Default: hug
  Trap: fixtures/grow-outside-host.bad.sail — `size(grow)` shares leftover inside a host (`<row>` `<column>` `<grid>` `<center>` `<stack>`). This element is not a child of a host.
        fixtures/flex-trap.bad.sail — Style error: `size(full)` is a DTCG CSS trap. Use `size(grow)` or `size(max)`.
        Language pointer only. Quoted below. Not compiled on this page.

        <column style({ size(grow) })>x</column>
        <row><column style({ size(full) })>x</column></row>
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style hugged = { size(hug), pad(3), bg(primary) }
  style leftover = { size(grow), pad(3), bg(accent) }
  style stretched = { size(max), pad(3), bg(muted) }
  style tall = { size({ h: 16 }) }
  style layer = { size({ w: 16, h: 8 }), bg(muted) }
  style rail = { size({ w: 24 }) }
</scope>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>row — hug + grow leftover</p>
    <p slot="spell" style(spell)>size(hug) / size(grow) in &lt;row&gt;</p>
    <row style(rail)>
      <column style(hugged)>hug</column>
      <column style(leftover)>grow</column>
    </row>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>row — leftover fills the stage</p>
    <p slot="spell" style(spell)>size(hug) / size(grow) · no w:24 pin</p>
    <row>
      <column style(hugged)>hug</column>
      <column style(leftover)>grow</column>
    </row>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>row — hug + max stretch</p>
    <p slot="spell" style(spell)>size(hug) / size(max)</p>
    <row style(rail)>
      <column style(hugged)>hug</column>
      <column style(stretched)>max</column>
    </row>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>column — hug + grow, host 16 tall</p>
    <p slot="spell" style(spell)>&lt;column&gt; size h:16</p>
    <column style(tall)>
      <row style(hugged)>hug</row>
      <row style(leftover)>grow</row>
    </column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>grid — hug cell + grow cell</p>
    <p slot="spell" style(spell)>&lt;grid cols="2"&gt;</p>
    <grid cols="2" gap="2">
      <column style(hugged)>hug</column>
      <column style(leftover)>grow</column>
    </grid>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>center — hug child</p>
    <p slot="spell" style(spell)>&lt;center&gt; size(hug)</p>
    <center>
      <column style(hugged)>hug</column>
    </center>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>stack — hug layer over a muted bar</p>
    <p slot="spell" style(spell)>&lt;stack&gt;</p>
    <stack>
      <column style(layer)></column>
      <column style(hugged)>hug</column>
    </stack>
  </l0-case>
</column>
