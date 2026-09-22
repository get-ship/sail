<!--
  Look: every gradient molecule on bg / fg / border.
  Spelling: bg({ gradient: primary }) · fg({ gradient: accent }) · border({ gradient: primary })
  Owner: language (gradient slot table)
  Trap: fixtures/gradient-unknown.bad.sail
-->
<column gap="6">
  <l0-case look="bg — primary · accent · surface (none wash) · none" spell="bg gradient: primary | accent | surface | none">
    <l0-grad-face kind="rail">
      <row gap="3">
        <l0-grad-face kind="bg-primary"></l0-grad-face>
        <l0-grad-face kind="bg-accent"></l0-grad-face>
        <l0-grad-face kind="bg-surface"></l0-grad-face>
        <l0-grad-face kind="bg-none"></l0-grad-face>
      </row>
    </l0-grad-face>
  </l0-case>
  <l0-case look="fg — gradient ink on title, same table" spell="fg gradient: primary | accent | surface | none">
    <l0-grad-face kind="rail">
      <row gap="3" align="center">
        <l0-grad-face kind="fg-primary">Aa</l0-grad-face>
        <l0-grad-face kind="fg-accent">Aa</l0-grad-face>
        <l0-grad-face kind="fg-surface">Aa</l0-grad-face>
        <l0-grad-face kind="fg-none">Aa</l0-grad-face>
      </row>
    </l0-grad-face>
  </l0-case>
  <l0-case look="border — 4-step stroke, same table" spell="border gradient: primary | accent | surface | none">
    <l0-grad-face kind="rail">
      <row gap="3">
        <l0-grad-face kind="bd-primary"></l0-grad-face>
        <l0-grad-face kind="bd-accent"></l0-grad-face>
        <l0-grad-face kind="bd-surface"></l0-grad-face>
        <l0-grad-face kind="bd-none"></l0-grad-face>
      </row>
    </l0-grad-face>
  </l0-case>
</column>
