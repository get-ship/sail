<!--
  <toast-region> — fixed stacking region. Width/reverse/insets via sx+CVA.
-->
<scope>
  props: {
    position: {
      'top-start': 'top-0 right-auto bottom-auto left-0',
      'top-end': 'top-0 right-0 bottom-auto left-auto',
      'bottom-start': 'top-auto right-auto bottom-0 left-0 col-reverse',
      'bottom-end': 'top-auto right-0 bottom-0 left-auto col-reverse',
    }
  }
</scope>
<column gap="2" sx="fixed z-(--z-toast) p-4 pointer-events-none top-0 right-auto bottom-auto left-0 w-panel-w-toast">
  <slot></slot>
</column>
