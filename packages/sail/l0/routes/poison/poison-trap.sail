<!--
  Registered CE is src/poison-trap/.
  Look: one painted box with page inset. The illegal sibling const
        did not unregister `base`.
  Spelling: style base = { pad(page) } ; <column style(base)>
  Owner: theme (`pad(page)`)
  Default: no inset.
  Trap: fixtures/poison-trap.bad.sail — Style `wide`: unknown vocabulary `size.wide`.
        Quoted below. Not compiled on this page (CI: pages compile;
        trap fixtures assert the error).

        <scope>
          style base = { pad(page) }
          style wide = { size(wide) }
        </scope>
        <column style(base)>x</column>
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style base = { pad(page) }
  style face = { bg(primary) }
</scope>
<column gap="3">
  <p style(look)>base survived — pad page still paints after the illegal sibling</p>
  <p style(spell)>style base = pad page · illegal sibling size.wide did not unregister base</p>
  <p style(spell)>trap: Style wide — unknown vocabulary size.wide</p>
  <column style(base)>
    <column style(face)>base survived</column>
  </column>
</column>
