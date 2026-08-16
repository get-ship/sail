<!--
  Lab canvas for HeaderSimple. Mount only — not a template.
  Source of truth: src/blocks/HeaderSimple/HeaderSimple.sail
-->
<scope>
  style canvas = { pad(6) }
</scope>

<column gap="3" style(canvas)>
  <small>HeaderSimple</small>
  <navbar bordered>
    <strong slot="brand">Mantine</strong>
    <row slot="center" gap="2">
      <a href="/about">Features</a>
      <a href="/pricing">Pricing</a>
      <a href="/learn">Learn</a>
      <a href="/community">Community</a>
    </row>
    <button slot="actions" variant="ghost" size="icon" label="Toggle navigation">
      <icon name="menu"></icon>
    </button>
  </navbar>
</column>
