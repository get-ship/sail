<scope>
  props: {
    look = ''
    spell = ''
    note = ''
  }
  style band = { size({ w: 100% }), pad(5), bg(muted), radius(md) }
  style pair = { size({ w: 100%, min-h: 32 }) }
  style well = { size(grow), pad(6), bg(surface), radius(sm) }
  style nums = { size({ w: 48 }), pad({ l: 4 }) }
  style lookType = { type(label), fg(muted) }
  style spellType = { type(mono), fg(muted) }
</scope>

<column gap="4" style(band)>
  <column gap="1">
    <if {look() !== ''}>
      <p style(lookType)>{look()}</p>
    </if>
    <if {spell() !== ''}>
      <p style(spellType)>{spell()}</p>
    </if>
    <if {note() !== ''}>
      <p style(spellType)>{note()}</p>
    </if>
  </column>
  <row gap="4" align="stretch" style(pair)>
    <column style(well)>
      <center>
        <slot></slot>
      </center>
    </column>
    <column gap="1" style(nums)>
    </column>
  </row>
</column>
