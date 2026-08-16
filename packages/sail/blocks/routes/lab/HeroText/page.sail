<!--
  Lab canvas for HeroText. Mount only — not a template.
  Source of truth: src/blocks/hero-text/hero-text.sail
  Do not wrap in <center>: measure-lg's max-width 100% is circular
  inside a grid item and the heading overflows at 390.
-->
<column gap="3">
  <small>HeroText</small>
  <hero-text></hero-text>
</column>
