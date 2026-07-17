<!--
  <toast-region position="bottom-end"> — L1 toast host (element mode). A fixed,
  stacking region that toasts are appended into. aria-live is on each <toast>, so
  the region is a plain landmark. Bare tag lowers to
  <div class="sail-toast-region">.

    <toast-region position="bottom-end">
      <toast tone="success">Saved.</toast>
    </toast-region>

  position: top-start / top-end / bottom-start / bottom-end (default).
-->

<scope>
  props: {
    position: 'top-start' | 'top-end' | 'bottom-start' | 'bottom-end' = 'top-start'
  }
</scope>

<style>
  .sail-toast-region {
    position: fixed;
    z-index: var(--z-toast, 1100);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    width: min(24rem, calc(100vw - var(--space-8)));
    padding: var(--space-4);
    pointer-events: none;
    inset-block-end: 0;
    inset-inline-end: 0;
  }
  .sail-toast-region[position='top-start'] { inset: 0 auto auto 0; }
  .sail-toast-region[position='top-end'] { inset: 0 0 auto auto; }
  .sail-toast-region[position='bottom-start'] { inset: auto auto 0 0; }
  .sail-toast-region[position='bottom-end'] { inset: auto 0 0 auto; }
  .sail-toast-region[position^='bottom'] { flex-direction: column-reverse; }
  .sail-toast-region > * { pointer-events: auto; }
</style>

<div><slot></slot></div>
