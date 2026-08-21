<scope>
  props: {
    look = ''
    spell = ''
    note = ''
  }
  style band = { size({ w: 100% }), pad(5), bg(surface), border({ width: 1, style: solid, color: muted }), radius(md) }
  style pair = { size(hug) }
  style well = { size(hug), pad(3) }
  style nums = { size(hug), pad({ l: 4 }) }
  style lookType = { type(label) }
  style spellType = { type(mono) }
</scope>

<column gap="3" style(band)>
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
  <row gap="4" align="start" style(pair)>
    <column style(well)>
      <slot></slot>
    </column>
    <column gap="1" style(nums)>
      <p style(spellType)>no legal bind — inView-shaped callback into a controller signal</p>
    </column>
  </row>
</column>
