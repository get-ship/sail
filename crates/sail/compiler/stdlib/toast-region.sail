<!--
  <toast-region> — fixed stacking region for <toast>s.

  Host: column+gap layout; fixed/z/pad/pointer-events + PE top-start inset on sx.
  CVA position matrix keys inset. Width min() formula, bottom* column-reverse,
  and child pointer-events auto are contracts.
-->

<scope>
  props: {
    // first key = default = PE top-start (attr-absent base on host sx)
    position: {
      'top-start': 'top-0 right-auto bottom-auto left-0',
      'top-end': 'top-0 right-0 bottom-auto left-auto',
      'bottom-start': 'top-auto right-auto bottom-0 left-0',
      'bottom-end': 'top-auto right-0 bottom-0 left-auto',
    }
  }
</scope>

<column gap="2" sx="fixed z-(--z-toast) p-4 pointer-events-none top-0 right-auto bottom-auto left-0">
  <slot></slot>
</column>
