<!--
  Look: a painted box; primary by default, accent at tablet, muted at desktop
  Spelling: when: { tablet, desktop }
  Owner: language keys, theme breakpoints
  Trap: fixtures/when-unknown-key.bad.sail
-->
<column gap="6">
  <l0-case look="64px box: primary, accent at tablet, muted at desktop" spell="when tablet / desktop · size w:16 h:16">
    <l0-when-face kind="stack"></l0-when-face>
  </l0-case>
  <l0-case look="mobile — primary at mobile, muted otherwise" spell="when mobile · bg(primary)">
    <l0-when-face kind="mobile"></l0-when-face>
  </l0-case>
  <l0-case look="tablet — accent at tablet, muted otherwise" spell="when tablet · bg(accent)">
    <l0-when-face kind="tablet"></l0-when-face>
  </l0-case>
  <l0-case look="desktop — primary at desktop, muted otherwise" spell="when desktop · bg(primary)">
    <l0-when-face kind="desktop"></l0-when-face>
  </l0-case>
</column>
