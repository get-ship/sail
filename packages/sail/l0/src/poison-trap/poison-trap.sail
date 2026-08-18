<!--
  Look: one painted box with page inset. The illegal sibling const
        did not unregister `base`.
  Spelling: style base = { pad(page) } ; <column style(base)>
  Owner: theme (`pad(page)`)
  Default: no inset.
  Trap: fixtures/poison-trap.bad.sail — Style `wide`: unknown vocabulary `wide`.
        Quoted below. Not compiled on this page (CI: pages compile;
        trap fixtures assert the error).

        <scope>
          style base = { pad(page) }
          style wide = { size(wide) }
        </scope>
        <column style(base)>x</column>
-->
<scope>
  style base = { pad(page) }
  style face = { bg(primary) }
</scope>
<column style(base)>
  <column style(face)>base survived</column>
</column>
