<!--
  HeroText — centered lead with two actions.
  size(measure-lg) is the section column (60ch) so the dek wraps.
  Do not wrap that column in <center>: max-width 100% goes circular
  in a grid item and the heading overflows at 390. Measure centers.
  Inner <center> sits the title; type(align) is illegal.
-->
<scope>
  style shell = { size(measure-lg) }
</scope>

<column gap="4" style(shell)>
  <center>
    <h1>Automated AI code reviews for any stack</h1>
  </center>
  <p>Build more reliable software with AI companion. AI is also trained to detect lazy developers who do nothing and just complain on Twitter.</p>
  <row gap="3" justify="center">
    <button variant="outline">Book a demo</button>
    <button>Purchase a license</button>
  </row>
</column>
