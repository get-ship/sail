<!--
  Lab canvas for HeroTitle. Mount only — not a template.
  Source of truth: src/blocks/hero-title/hero-title.sail
  Do not wrap in <center>: measure-lg's max-width 100% is circular
  inside a grid item and the heading overflows at 390.
-->
<column gap="3">
  <small>HeroTitle</small>
  <hero-title></hero-title>
</column>
