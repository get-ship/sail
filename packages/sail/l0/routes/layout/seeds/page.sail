<!--
  Look: named seed cases as colored boxes.
  Spelling: one cell per seed, hosts + participation + free size slots.
  Owner: language (structure) + free geometry
-->
<column gap="6">
  <l0-case look="grow-without-bound — grow child, host has no width" spell="size(grow) in &lt;row&gt;">
    <row inline>
      <l0-seed-face kind="leftover">grow</l0-seed-face>
    </row>
  </l0-case>
  <l0-case look="circular-percentage — child 50% of a 50% parent" spell="size w:50% nested">
    <l0-seed-face kind="pct-parent">
      <l0-seed-face kind="pct-child">pct</l0-seed-face>
    </l0-seed-face>
  </l0-case>
  <l0-case look="RTL wrap — dir=rtl, 8px gap" spell="&lt;row dir=rtl gap=2&gt;">
    <row dir="rtl" gap="2">
      <l0-seed-face kind="box4"></l0-seed-face>
      <l0-seed-face kind="box4a"></l0-seed-face>
      <l0-seed-face kind="box4m"></l0-seed-face>
    </row>
  </l0-case>
  <l0-case look="sibling-overlap opt-in — stack overlays" spell="&lt;stack&gt;">
    <stack>
      <l0-seed-face kind="overlap-a"></l0-seed-face>
      <l0-seed-face kind="overlap-b"></l0-seed-face>
    </stack>
  </l0-case>
  <l0-case look="holy-grail — header / nav+main+aside / footer" spell="column of rows · nav/aside w:16 · main grow">
    <l0-seed-face kind="fill">
      <l0-seed-face kind="header">header</l0-seed-face>
      <row nowrap>
        <l0-seed-face kind="nav">nav</l0-seed-face>
        <l0-seed-face kind="main">main</l0-seed-face>
        <l0-seed-face kind="aside">aside</l0-seed-face>
      </row>
      <l0-seed-face kind="footer">footer</l0-seed-face>
    </l0-seed-face>
  </l0-case>
  <l0-case look="sidebar-fixed-fluid — 16-step rail + grow" spell="size w:16 + size(grow)">
    <l0-seed-face kind="fill">
      <row nowrap>
        <l0-seed-face kind="rail">rail</l0-seed-face>
        <l0-seed-face kind="fluid">fluid</l0-seed-face>
      </row>
    </l0-seed-face>
  </l0-case>
  <l0-case look="bento spanning — col-span / row-span on 4 tracks" spell="&lt;grid cols=4&gt; col-span=2 row-span=2">
    <grid cols="4" gap="2">
      <column col-span="2" row-span="2">
        <l0-seed-face kind="span-a">a</l0-seed-face>
      </column>
      <l0-seed-face kind="span-b">b</l0-seed-face>
      <l0-seed-face kind="span-c">c</l0-seed-face>
      <column col-span="2">
        <l0-seed-face kind="span-b">d</l0-seed-face>
      </column>
    </grid>
  </l0-case>
  <l0-case look="aspect-vs-grow — leftover width, height matches" spell="size(grow) + size({ aspect: square })">
    <row>
      <l0-seed-face kind="aspect-grow"></l0-seed-face>
    </row>
  </l0-case>
</column>
