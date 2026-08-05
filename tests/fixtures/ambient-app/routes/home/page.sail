<!--
  ambient-app home — fixture surface for plain tsc + ambient pack proof.
  Controller is a sibling module (extracted shape) so tsc can see authored TS
  without compiling the template (two-checker: sail-compiler owns the template).
-->
<main>
  <h1>ambient app</h1>
  <p>count: {count()}</p>
  <button on:click={inc}>inc</button>
</main>
