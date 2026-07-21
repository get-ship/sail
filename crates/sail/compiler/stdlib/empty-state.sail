<!--
  <empty-state> — L1 language element (element mode). A centered placeholder for
  empty lists / missing content: an icon tile, a title, description copy, and an
  action row. Named slots (icon / title / action + the default description slot)
  project at compile time; bare tag lowers to <div class="sail-empty-state">.

  Faithful to the feat/next sail-empty-state baseline. Shadow-isms dissolved at L1:
  - host/.box split → collapsed onto the single root (the root IS the box).
  - reactive align={variant==='inline'?...} on the copy column → CSS variant rule.
  - reflect-empty="data-no-icon" (a JS slot-emptiness reflect) → `.icon:empty`
    (pure CSS — an empty icon slot leaves the tile empty, so it hides itself).
  - reactive role/aria-live (only the `connecting` variant is a live region) →
    the use:variantLive capability, off a static role=note default.
  Icon tile tints per tone from the global [tone] channel; default/muted stay
  neutral (empty states read quiet by default).
-->

<scope>
  props: {
    variant: 'default' | 'inline' | 'placeholder' | 'connecting' = 'default'
    size: 'sm' | 'md' | 'lg' = 'md'
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
  }
</scope>

<div use:variantLive role="note" aria-live="off">
  <div class="icon" aria-hidden="true"><slot name="icon"></slot></div>
  <div class="copy">
    <div class="title"><slot name="title"></slot></div>
    <div class="desc"><slot></slot></div>
  </div>
  <div class="actions"><slot name="action"></slot></div>
</div>
