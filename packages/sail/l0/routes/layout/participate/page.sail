<!--
  Look: grow / hug / max per host.
  Spelling: size(hug) / size(grow) / size(max) on children of hosts.
  Owner: language (participation)
  Default: hug
  Trap: fixtures/grow-outside-host.bad.sail · fixtures/flex-trap.bad.sail
-->
<column gap="6">
  <l0-case look="row — hug + grow leftover" spell="size(hug) / size(grow) in &lt;row&gt;">
    <l0-rail>
      <l0-part kind="hug">hug</l0-part>
      <l0-part kind="grow">grow</l0-part>
    </l0-rail>
  </l0-case>
  <l0-case look="row — leftover fills the stage" spell="size(hug) / size(grow) · no w:24 pin">
    <row>
      <l0-part kind="hug">hug</l0-part>
      <l0-part kind="grow">grow</l0-part>
    </row>
  </l0-case>
  <l0-case look="row — hug + max stretch" spell="size(hug) / size(max)">
    <l0-rail>
      <l0-part kind="hug">hug</l0-part>
      <l0-part kind="max">max</l0-part>
    </l0-rail>
  </l0-case>
  <l0-case look="column — hug + grow, host 16 tall" spell="&lt;column&gt; size h:16">
    <l0-tall>
      <l0-part kind="hug">hug</l0-part>
      <l0-part kind="grow">grow</l0-part>
    </l0-tall>
  </l0-case>
  <l0-case look="grid — hug cell + grow cell" spell="&lt;grid cols=2&gt;">
    <grid cols="2" gap="2">
      <l0-part kind="hug">hug</l0-part>
      <l0-part kind="grow">grow</l0-part>
    </grid>
  </l0-case>
  <l0-case look="center — hug child" spell="&lt;center&gt; size(hug)">
    <center>
      <l0-part kind="hug">hug</l0-part>
    </center>
  </l0-case>
  <l0-case look="stack — hug layer over a muted bar" spell="&lt;stack&gt;">
    <stack>
      <l0-swatch kind="bar" face="muted"></l0-swatch>
      <l0-part kind="hug">hug</l0-part>
    </stack>
  </l0-case>
</column>
