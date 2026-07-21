<!--
  <prose> — typography scope for rendered content (markdown blobs etc.).

  Host: block + body typeface/color/leading via sx.
  Contracts: all descendant rules (headings, p, a, code/pre, blockquote, lists,
  table, strong, ::marker) — child-keyed typography recipe.
-->

<scope>
</scope>

<div sx="block font-body text-foreground leading-normal-plus"><slot></slot></div>
