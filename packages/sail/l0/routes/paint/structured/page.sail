<!--
  Look: a painted box whose face is a token via {ref}, and a mix of two tokens via {op: mix}.
  Owner: language (structured {ref} / {op})
  Trap: fixtures/structured-unknown.bad.sail · structured-dangling
-->
<column gap="6">
  <l0-case look="ref — primary face from a token ref" spell="bg ref: primary">
    <l0-struct-face kind="ref-bg"></l0-struct-face>
  </l0-case>
  <l0-case look="ref in a color slot — accent ink" spell="fg color: ref accent">
    <l0-struct-face kind="ref-fg">Aa</l0-struct-face>
  </l0-case>
  <l0-case look="op mix — primary 50% onto accent" spell="bg op: mix from primary onto accent amount 50">
    <l0-struct-face kind="mix"></l0-struct-face>
  </l0-case>
  <l0-case look="chain — pad + muted face + primary ink, both refs" spell="pad 4 · bg ref muted · fg ref primary">
    <l0-struct-face kind="chain">Aa</l0-struct-face>
  </l0-case>
</column>
