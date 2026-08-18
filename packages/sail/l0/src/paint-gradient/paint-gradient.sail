<!--
  Look: every gradient molecule on bg / fg (text) / border.
        primary and accent wash the face. surface is none on sail —
        fg surface keeps authored ink (does not clip to none).
        none clears the wash. Text clip reads as gradient ink.
  Spelling: bg({ gradient: primary }) · fg({ gradient: accent }) · border({ gradient: primary })
  Owner: language (gradient slot table: primary | accent | surface | none)
  Default: no wash
  Trap: fixtures/gradient-unknown.bad.sail — Style error: rainbow is not a gradient molecule: primary | accent | surface | none
        Quoted below. Not compiled on this page.

        <column style({ bg({ gradient: rainbow }) })>x</column>
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style rail = { size({ w: 100%, h: 24 }) }
  style bgPrimary = { size(grow), radius(sm), bg({ gradient: primary }) }
  style bgAccent = { size(grow), radius(sm), bg({ gradient: accent }) }
  style bgSurface = { size(grow), radius(sm), bg({ color: muted, gradient: surface }) }
  style bgNone = { size(grow), radius(sm), bg({ color: muted, gradient: none }) }
  style fgPrimary = { size(grow), radius(sm), type(heading), fg({ gradient: primary }) }
  style fgAccent = { size(grow), radius(sm), type(heading), fg({ gradient: accent }) }
  style fgSurface = { size(grow), radius(sm), type(heading), fg({ color: muted, gradient: surface }) }
  style fgNone = { size(grow), radius(sm), type(heading), fg({ color: muted, gradient: none }) }
  style bdPrimary = { size(grow), radius(sm), bg(surface), border({ width: 4, style: solid, gradient: primary }) }
  style bdAccent = { size(grow), radius(sm), bg(surface), border({ width: 4, style: solid, gradient: accent }) }
  style bdSurface = { size(grow), radius(sm), bg(surface), border({ width: 4, style: solid, color: muted, gradient: surface }) }
  style bdNone = { size(grow), radius(sm), bg(surface), border({ width: 4, style: solid, color: muted, gradient: none }) }
</scope>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>bg — primary · accent · surface (none wash) · none</p>
    <p slot="spell" style(spell)>bg gradient: primary | accent | surface | none</p>
    <row gap="3" style(rail)>
      <column style(bgPrimary)></column>
      <column style(bgAccent)></column>
      <column style(bgSurface)></column>
      <column style(bgNone)></column>
    </row>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>fg — gradient ink on title, same table</p>
    <p slot="spell" style(spell)>fg gradient: primary | accent | surface | none</p>
    <row gap="3" align="center" style(rail)>
      <column style(fgPrimary)>Aa</column>
      <column style(fgAccent)>Aa</column>
      <column style(fgSurface)>Aa</column>
      <column style(fgNone)>Aa</column>
    </row>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>border — 4-step stroke, same table</p>
    <p slot="spell" style(spell)>border gradient: primary | accent | surface | none</p>
    <row gap="3" style(rail)>
      <column style(bdPrimary)></column>
      <column style(bdAccent)></column>
      <column style(bdSurface)></column>
      <column style(bdNone)></column>
    </row>
  </l0-case>
</column>
