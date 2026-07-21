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

<column gap="2" sx="fixed z-(--z-toast) p-4 pointer-events-none"><slot></slot></column>
