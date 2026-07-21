<!--
  <toast-region position="bottom-end"> — fixed stacking region for <toast>s.

  Host paint: column+gap layout; fixed/z/pad/pointer-events on sx.
  Residue: width min() formula, logical inset position matrix, column-reverse
  for bottom*, child pointer-events auto.
-->

<scope>
  props: {
    position: 'top-start' | 'top-end' | 'bottom-start' | 'bottom-end' = 'top-start'
  }
</scope>

<style>
  .sail-toast-region {
    width: min(24rem, calc(100vw - var(--space-8)));
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

<column gap="2" sx="fixed z-(--z-toast) p-4 pointer-events-none"><slot></slot></column>
