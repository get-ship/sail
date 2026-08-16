<!--
  <page> — the document. Not chrome. Not a measure. Not a max-width box.

  A column is a box. A page is a kind: this view is a document.
  Children stack. Theme inset so the document never kisses the glass.
  Width is the viewport. Measure belongs to the pieces (prose, form, a card).
  Layouts name this host. A rail opts out.
-->

<scope>
  style insetMd = { pad(page-md) }
  style insetLg = { pad(page-lg) }
  style base = { pad(page), when: { tablet: insetMd, desktop: insetLg } }
</scope>

<column gap="6" style(base)>
  <slot />
  <slot name="route" />
</column>
