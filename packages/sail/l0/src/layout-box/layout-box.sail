<!--
  Look: one painted box of free width 7
  Spelling: style box = { size({ w: 7 }), bg(accent) }
  Owner: free
  Default: hug content
  Trap: fixtures/size-wide.bad.sail — Style error: unknown vocabulary `size.wide`
        Quoted below. Not compiled on this page.

        <column style({ size(wide) })>x</column>
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style box = { size({ w: 7, h: 8 }), bg(accent) }
  style onlyW = { size({ w: 16, h: 8 }), bg(primary) }
  style onlyH = { size({ w: 8, h: 16 }), bg(primary) }
  style minW = { size({ min-w: 16, h: 8 }), bg(accent) }
  style minH = { size({ w: 8, min-h: 16 }), bg(accent) }
  style maxW = { size({ max-w: 16, h: 8 }), bg(muted) }
  style maxH = { size({ w: 16, max-h: 8 }), bg(muted) }
  style square = { size({ w: 16, aspect: square }), bg(primary) }
  style video = { size({ w: 16, aspect: video }), bg(accent) }
  style auto = { size({ w: 16, h: 8, aspect: auto }), bg(muted) }
  style zero = { size({ w: 0, h: 0 }), bg(primary) }
  style clamp = { size({ max-w: 16 }), overflow(hidden), bg(muted) }
  style huge = { size({ w: 48, h: 8 }), bg(accent) }
</scope>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>free width 7 (28px), height 8 (32px)</p>
    <p slot="spell" style(spell)>size w:7 h:8 · bg(accent)</p>
    <column style(box)></column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>w — 16-step bar</p>
    <p slot="spell" style(spell)>size w:16 h:8</p>
    <column style(onlyW)></column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>h — 16-step tall</p>
    <p slot="spell" style(spell)>size w:8 h:16</p>
    <column style(onlyH)></column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>min-w — at least 16 steps</p>
    <p slot="spell" style(spell)>size min-w:16 h:8</p>
    <column style(minW)></column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>min-h — at least 16 steps</p>
    <p slot="spell" style(spell)>size w:8 min-h:16</p>
    <column style(minH)></column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>max-w — at most 16 steps</p>
    <p slot="spell" style(spell)>size max-w:16 h:8</p>
    <column style(maxW)>max-w 16</column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>max-h — at most 8 steps</p>
    <p slot="spell" style(spell)>size w:16 max-h:8</p>
    <column style(maxH)>max-h 8</column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>aspect square</p>
    <p slot="spell" style(spell)>size w:16 aspect:square</p>
    <column style(square)></column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>aspect video</p>
    <p slot="spell" style(spell)>size w:16 aspect:video</p>
    <column style(video)></column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>aspect auto</p>
    <p slot="spell" style(spell)>size w:16 h:8 aspect:auto</p>
    <column style(auto)></column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>zero box</p>
    <p slot="spell" style(spell)>size w:0 h:0</p>
    <column style(zero)></column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>16-step clamp, 48-step child</p>
    <p slot="spell" style(spell)>size max-w:16 overflow(hidden) · child w:48</p>
    <column style(clamp)>
      <column style(huge)></column>
    </column>
  </l0-case>
</column>
