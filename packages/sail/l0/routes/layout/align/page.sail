<!--
  Look: three 4-step boxes at the bottom of a 24×16 rail
  Spelling: <row align=end gap=2>
  Owner: language (align / justify attrs)
  Default: row align is center; row/stack justify is start; column justify has no default
  Trap: fixtures/align-off-enum.bad.sail
-->
<column gap="6">
  <l0-case look="three 4-step boxes at the bottom of a 24×16 rail" spell="&lt;row align=end gap=2&gt;">
    <l0-align-face>
      <row align="end" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </row>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes at the top of a 24×16 rail" spell="&lt;row align=start gap=2&gt;">
    <l0-align-face>
      <row align="start" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </row>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes vertically centered in a 24×16 rail" spell="&lt;row align=center gap=2&gt;">
    <l0-align-face>
      <row align="center" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </row>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 8-step-wide boxes stretched to the height of a 24×16 rail" spell="&lt;row align=stretch&gt;">
    <l0-align-face>
      <row align="stretch">
        <l0-swatch kind="stretch"></l0-swatch>
        <l0-swatch kind="stretch" face="accent"></l0-swatch>
        <l0-swatch kind="stretch" face="muted"></l0-swatch>
      </row>
    </l0-align-face>
  </l0-case>
  <l0-case look="three type boxes on one baseline in a 24×16 rail" spell="&lt;row align=baseline&gt;">
    <l0-align-face>
      <row align="baseline">
        <l0-align-base kind="even">Ay</l0-align-base>
        <l0-align-base face="accent" kind="high">Hg</l0-align-base>
        <l0-align-base face="muted" kind="low">x</l0-align-base>
      </row>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes packed at the start of a 24×16 rail" spell="&lt;row justify=start gap=2&gt;">
    <l0-align-face>
      <row justify="start" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </row>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes centered on a 24×16 rail" spell="&lt;row justify=center gap=2&gt;">
    <l0-align-face>
      <row justify="center" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </row>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes packed at the end of a 24×16 rail" spell="&lt;row justify=end gap=2&gt;">
    <l0-align-face>
      <row justify="end" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </row>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes, leftover split between them" spell="&lt;row justify=between gap=2&gt;">
    <l0-align-face>
      <row justify="between" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </row>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes, leftover around them" spell="&lt;row justify=around gap=2&gt;">
    <l0-align-face>
      <row justify="around" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </row>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes, leftover even on both sides and between" spell="&lt;row justify=evenly gap=2&gt;">
    <l0-align-face>
      <row justify="evenly" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </row>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes packed at the top of a 24×16 well" spell="&lt;column justify=start gap=2&gt;">
    <l0-align-face>
      <column justify="start" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </column>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes centered in a 24×16 well" spell="&lt;column justify=center gap=2&gt;">
    <l0-align-face>
      <column justify="center" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </column>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes packed at the bottom of a 24×16 well" spell="&lt;column justify=end gap=2&gt;">
    <l0-align-face>
      <column justify="end" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </column>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes, leftover split between them in a column" spell="&lt;column justify=between gap=2&gt;">
    <l0-align-face>
      <column justify="between" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </column>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes, leftover around them in a column" spell="&lt;column justify=around gap=2&gt;">
    <l0-align-face>
      <column justify="around" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </column>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes, leftover even in a column" spell="&lt;column justify=evenly gap=2&gt;">
    <l0-align-face>
      <column justify="evenly" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </column>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes at the left of a 24×16 well" spell="&lt;column align=start gap=2&gt;">
    <l0-align-face>
      <column align="start" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </column>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes centered across a 24×16 well" spell="&lt;column align=center gap=2&gt;">
    <l0-align-face>
      <column align="center" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </column>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes at the right of a 24×16 well" spell="&lt;column align=end gap=2&gt;">
    <l0-align-face>
      <column align="end" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </column>
    </l0-align-face>
  </l0-case>
  <l0-case look="two 8-step-tall boxes stretched across a 24×16 well" spell="&lt;column align=stretch&gt;">
    <l0-align-face>
      <column align="stretch">
        <l0-swatch kind="cell"></l0-swatch>
        <l0-swatch kind="cell" face="accent"></l0-swatch>
      </column>
    </l0-align-face>
  </l0-case>
  <l0-case look="type boxes stacked on one baseline in a 24×16 well" spell="&lt;column align=baseline&gt;">
    <l0-align-face>
      <column align="baseline">
        <l0-align-base kind="even">Ay</l0-align-base>
        <l0-align-base face="accent" kind="high">Hg</l0-align-base>
        <l0-align-base face="muted" kind="low">x</l0-align-base>
      </column>
    </l0-align-face>
  </l0-case>
  <l0-case look="muted bar under a short box, packed at the start of a 24×16 rail" spell="&lt;stack justify=start&gt;">
    <l0-align-face>
      <stack justify="start">
        <l0-swatch kind="bar" face="muted"></l0-swatch>
        <l0-swatch kind="short"></l0-swatch>
      </stack>
    </l0-align-face>
  </l0-case>
  <l0-case look="muted bar under a short box, centered on a 24×16 rail" spell="&lt;stack justify=center&gt;">
    <l0-align-face>
      <stack justify="center">
        <l0-swatch kind="bar" face="muted"></l0-swatch>
        <l0-swatch kind="short"></l0-swatch>
      </stack>
    </l0-align-face>
  </l0-case>
  <l0-case look="muted bar under a short box, packed at the end of a 24×16 rail" spell="&lt;stack justify=end&gt;">
    <l0-align-face>
      <stack justify="end">
        <l0-swatch kind="bar" face="muted"></l0-swatch>
        <l0-swatch kind="short"></l0-swatch>
      </stack>
    </l0-align-face>
  </l0-case>
  <l0-case look="muted bar under a short box, leftover split between layers" spell="&lt;stack justify=between&gt;">
    <l0-align-face>
      <stack justify="between">
        <l0-swatch kind="bar" face="muted"></l0-swatch>
        <l0-swatch kind="short"></l0-swatch>
      </stack>
    </l0-align-face>
  </l0-case>
  <l0-case look="muted bar under a short box, leftover around layers" spell="&lt;stack justify=around&gt;">
    <l0-align-face>
      <stack justify="around">
        <l0-swatch kind="bar" face="muted"></l0-swatch>
        <l0-swatch kind="short"></l0-swatch>
      </stack>
    </l0-align-face>
  </l0-case>
  <l0-case look="muted bar under a short box, leftover even around layers" spell="&lt;stack justify=evenly&gt;">
    <l0-align-face>
      <stack justify="evenly">
        <l0-swatch kind="bar" face="muted"></l0-swatch>
        <l0-swatch kind="short"></l0-swatch>
      </stack>
    </l0-align-face>
  </l0-case>
  <l0-case look="muted bar under a short box, aligned to the start of a 24×16 rail" spell="&lt;stack align=start&gt;">
    <l0-align-face>
      <stack align="start">
        <l0-swatch kind="bar" face="muted"></l0-swatch>
        <l0-swatch kind="short"></l0-swatch>
      </stack>
    </l0-align-face>
  </l0-case>
  <l0-case look="muted bar under a short box, aligned to the center of a 24×16 rail" spell="&lt;stack align=center&gt;">
    <l0-align-face>
      <stack align="center">
        <l0-swatch kind="bar" face="muted"></l0-swatch>
        <l0-swatch kind="short"></l0-swatch>
      </stack>
    </l0-align-face>
  </l0-case>
  <l0-case look="muted bar under a short box, aligned to the end of a 24×16 rail" spell="&lt;stack align=end&gt;">
    <l0-align-face>
      <stack align="end">
        <l0-swatch kind="bar" face="muted"></l0-swatch>
        <l0-swatch kind="short"></l0-swatch>
      </stack>
    </l0-align-face>
  </l0-case>
  <l0-case look="muted bar under a short box, stretched on a 24×16 rail" spell="&lt;stack align=stretch&gt;">
    <l0-align-face>
      <stack align="stretch">
        <l0-swatch kind="bar" face="muted"></l0-swatch>
        <l0-swatch kind="short"></l0-swatch>
      </stack>
    </l0-align-face>
  </l0-case>
  <l0-case look="muted bar under a short box, baseline-aligned on a 24×16 rail" spell="&lt;stack align=baseline&gt;">
    <l0-align-face>
      <stack align="baseline">
        <l0-align-base kind="even">Ay</l0-align-base>
        <l0-align-base face="accent" kind="high">Hg</l0-align-base>
      </stack>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes at the bottom, packed at the start of a 24×16 rail" spell="&lt;row align=end justify=start gap=2&gt;">
    <l0-align-face>
      <row align="end" justify="start" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </row>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes at the bottom, centered on a 24×16 rail" spell="&lt;row align=end justify=center gap=2&gt;">
    <l0-align-face>
      <row align="end" justify="center" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </row>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes at the bottom, packed at the end of a 24×16 rail" spell="&lt;row align=end justify=end gap=2&gt;">
    <l0-align-face>
      <row align="end" justify="end" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </row>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes at the bottom, leftover split between them" spell="&lt;row align=end justify=between gap=2&gt;">
    <l0-align-face>
      <row align="end" justify="between" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </row>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes at the bottom, leftover around them" spell="&lt;row align=end justify=around gap=2&gt;">
    <l0-align-face>
      <row align="end" justify="around" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </row>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes at the bottom, leftover even on both sides and between" spell="&lt;row align=end justify=evenly gap=2&gt;">
    <l0-align-face>
      <row align="end" justify="evenly" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </row>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes at the top, centered on a 24×16 rail" spell="&lt;row align=start justify=center gap=2&gt;">
    <l0-align-face>
      <row align="start" justify="center" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </row>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes vertically and horizontally centered on a 24×16 rail" spell="&lt;row align=center justify=center gap=2&gt;">
    <l0-align-face>
      <row align="center" justify="center" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </row>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 8-step-wide boxes stretched tall, leftover split between them" spell="&lt;row align=stretch justify=between&gt;">
    <l0-align-face>
      <row align="stretch" justify="between">
        <l0-swatch kind="stretch"></l0-swatch>
        <l0-swatch kind="stretch" face="accent"></l0-swatch>
        <l0-swatch kind="stretch" face="muted"></l0-swatch>
      </row>
    </l0-align-face>
  </l0-case>
  <l0-case look="three type boxes on one baseline, leftover around them" spell="&lt;row align=baseline justify=around&gt;">
    <l0-align-face>
      <row align="baseline" justify="around">
        <l0-align-base kind="even">Ay</l0-align-base>
        <l0-align-base face="accent" kind="high">Hg</l0-align-base>
        <l0-align-base face="muted" kind="low">x</l0-align-base>
      </row>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes at the right, packed at the top of a 24×16 well" spell="&lt;column align=end justify=start gap=2&gt;">
    <l0-align-face>
      <column align="end" justify="start" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </column>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes at the right, centered in a 24×16 well" spell="&lt;column align=end justify=center gap=2&gt;">
    <l0-align-face>
      <column align="end" justify="center" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </column>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes at the right, packed at the bottom of a 24×16 well" spell="&lt;column align=end justify=end gap=2&gt;">
    <l0-align-face>
      <column align="end" justify="end" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </column>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes at the right, leftover split in a column" spell="&lt;column align=end justify=between gap=2&gt;">
    <l0-align-face>
      <column align="end" justify="between" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </column>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes at the right, leftover around them in a column" spell="&lt;column align=end justify=around gap=2&gt;">
    <l0-align-face>
      <column align="end" justify="around" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </column>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes at the right, leftover even in a column" spell="&lt;column align=end justify=evenly gap=2&gt;">
    <l0-align-face>
      <column align="end" justify="evenly" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </column>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes at the left, centered in a 24×16 well" spell="&lt;column align=start justify=center gap=2&gt;">
    <l0-align-face>
      <column align="start" justify="center" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </column>
    </l0-align-face>
  </l0-case>
  <l0-case look="three 4-step boxes centered across and down a 24×16 well" spell="&lt;column align=center justify=center gap=2&gt;">
    <l0-align-face>
      <column align="center" justify="center" gap="2">
        <l0-swatch></l0-swatch>
        <l0-swatch face="accent"></l0-swatch>
        <l0-swatch face="muted"></l0-swatch>
      </column>
    </l0-align-face>
  </l0-case>
  <l0-case look="two 8-step-tall boxes stretched across, leftover split in a column" spell="&lt;column align=stretch justify=between&gt;">
    <l0-align-face>
      <column align="stretch" justify="between">
        <l0-swatch kind="cell"></l0-swatch>
        <l0-swatch kind="cell" face="accent"></l0-swatch>
      </column>
    </l0-align-face>
  </l0-case>
  <l0-case look="type boxes stacked on one baseline, leftover around them" spell="&lt;column align=baseline justify=around&gt;">
    <l0-align-face>
      <column align="baseline" justify="around">
        <l0-align-base kind="even">Ay</l0-align-base>
        <l0-align-base face="accent" kind="high">Hg</l0-align-base>
        <l0-align-base face="muted" kind="low">x</l0-align-base>
      </column>
    </l0-align-face>
  </l0-case>
  <l0-case look="muted bar under a short box, start/start on a 24×16 rail" spell="&lt;stack align=start justify=start&gt;">
    <l0-align-face>
      <stack align="start" justify="start">
        <l0-swatch kind="bar" face="muted"></l0-swatch>
        <l0-swatch kind="short"></l0-swatch>
      </stack>
    </l0-align-face>
  </l0-case>
  <l0-case look="muted bar under a short box, start/center on a 24×16 rail" spell="&lt;stack align=start justify=center&gt;">
    <l0-align-face>
      <stack align="start" justify="center">
        <l0-swatch kind="bar" face="muted"></l0-swatch>
        <l0-swatch kind="short"></l0-swatch>
      </stack>
    </l0-align-face>
  </l0-case>
  <l0-case look="muted bar under a short box, start/end on a 24×16 rail" spell="&lt;stack align=start justify=end&gt;">
    <l0-align-face>
      <stack align="start" justify="end">
        <l0-swatch kind="bar" face="muted"></l0-swatch>
        <l0-swatch kind="short"></l0-swatch>
      </stack>
    </l0-align-face>
  </l0-case>
  <l0-case look="muted bar under a short box, center/center on a 24×16 rail" spell="&lt;stack align=center justify=center&gt;">
    <l0-align-face>
      <stack align="center" justify="center">
        <l0-swatch kind="bar" face="muted"></l0-swatch>
        <l0-swatch kind="short"></l0-swatch>
      </stack>
    </l0-align-face>
  </l0-case>
  <l0-case look="muted bar under a short box, end/end on a 24×16 rail" spell="&lt;stack align=end justify=end&gt;">
    <l0-align-face>
      <stack align="end" justify="end">
        <l0-swatch kind="bar" face="muted"></l0-swatch>
        <l0-swatch kind="short"></l0-swatch>
      </stack>
    </l0-align-face>
  </l0-case>
  <l0-case look="muted bar under a short box, stretch/between on a 24×16 rail" spell="&lt;stack align=stretch justify=between&gt;">
    <l0-align-face>
      <stack align="stretch" justify="between">
        <l0-swatch kind="bar" face="muted"></l0-swatch>
        <l0-swatch kind="short"></l0-swatch>
      </stack>
    </l0-align-face>
  </l0-case>
  <l0-case look="type boxes overlaid on one baseline, leftover around them" spell="&lt;stack align=baseline justify=around&gt;">
    <l0-align-face>
      <stack align="baseline" justify="around">
        <l0-align-base kind="even">Ay</l0-align-base>
        <l0-align-base face="accent" kind="high">Hg</l0-align-base>
      </stack>
    </l0-align-face>
  </l0-case>
  <l0-case look="muted bar under a short box, end/evenly on a 24×16 rail" spell="&lt;stack align=end justify=evenly&gt;">
    <l0-align-face>
      <stack align="end" justify="evenly">
        <l0-swatch kind="bar" face="muted"></l0-swatch>
        <l0-swatch kind="short"></l0-swatch>
      </stack>
    </l0-align-face>
  </l0-case>
</column>
