<!--
  HeroTitle — display heading, lead, pair of actions.
  size({ w: 60ch, max-w: 100% }) is the section column. An h1 without it
  spans the viewport (fillRatio 1).
-->
<scope>
  style shell = { size({ w: 60ch, max-w: 100% }).margin({ x: auto }) }
</scope>

<column gap="6" style(shell)>
  <h1>A language for product UI</h1>
  <p>Ship is components, themes, and a compiler that refuses the wrong Style. Defaults hold so you do not paint every tag.</p>
  <row gap="3">
    <button>Get started</button>
    <a href="#" variant="default">GitHub</a>
  </row>
</column>
