<!--
  Look: three boxes in a row, aligned to the end, none stretch
  Spelling: <row align=end gap=2>
  Owner: language (align / justify attrs)
  Default: row align is center; row/stack justify is start; column justify has no default
  Trap: fixtures/align-off-enum.bad.sail
-->
<column gap="6">
  <l0-case look="three bars on a 16-step rail, align=end, 8px between" spell="&lt;row align=end gap=2&gt;">
    <l0-cross align="end">
      <l0-swatch kind="short"></l0-swatch>
      <l0-swatch kind="mid" face="accent"></l0-swatch>
      <l0-swatch kind="short" face="muted"></l0-swatch>
    </l0-cross>
  </l0-case>
  <l0-case look="align=start" spell="&lt;row align=start gap=2&gt;">
    <l0-cross align="start">
      <l0-swatch kind="short"></l0-swatch>
      <l0-swatch kind="mid" face="accent"></l0-swatch>
      <l0-swatch kind="short" face="muted"></l0-swatch>
    </l0-cross>
  </l0-case>
  <l0-case look="align=center" spell="&lt;row align=center gap=2&gt;">
    <l0-cross align="center">
      <l0-swatch kind="short"></l0-swatch>
      <l0-swatch kind="mid" face="accent"></l0-swatch>
      <l0-swatch kind="short" face="muted"></l0-swatch>
    </l0-cross>
  </l0-case>
  <l0-case look="align=stretch" spell="&lt;row align=stretch gap=2&gt;">
    <l0-cross align="stretch">
      <l0-swatch kind="stretch"></l0-swatch>
      <l0-swatch kind="stretch" face="accent"></l0-swatch>
      <l0-swatch kind="stretch" face="muted"></l0-swatch>
    </l0-cross>
  </l0-case>
  <l0-case look="align=baseline" spell="&lt;row align=baseline gap=2&gt;">
    <l0-cross align="baseline">
      <l0-align-base kind="even">Ay</l0-align-base>
      <l0-align-base face="accent" kind="high">Hg</l0-align-base>
      <l0-align-base face="muted" kind="low">x</l0-align-base>
    </l0-cross>
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
</column>
