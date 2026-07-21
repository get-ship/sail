<!--
  <prose> — typography scope for rendered content (markdown blobs etc.).

  Host paint: block + body typeface/color/leading via sx.
  Residue: all descendant rules (headings, p, a, code/pre, blockquote, lists,
  table, strong, ::marker) + specific sizes/margins/spacings.
-->

<scope>
</scope>

<div sx="block font-body text-foreground leading-normal-plus"><slot></slot></div>
