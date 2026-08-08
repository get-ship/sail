<!--
  <toast-region> — fixed stacking region for toasts.
  Position prop union values are CVA reflection only (Style match owns paint).
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

  style base = {
    box: position(fixed).pad(4).top(0).right(auto).bottom(auto).left(0).w(panel-w-toast),
    paint: z(z-toast).pointerEvents(none),
  }
</scope>

<column
  gap="2"
  style(base, {
    match: {
      position: {
        top-start: { box: top(0).right(auto).bottom(auto).left(0) },
        top-end: { box: top(0).right(0).bottom(auto).left(auto) },
        bottom-start: { box: top(auto).right(auto).bottom(0).left(0).flexDir(column-reverse) },
        bottom-end: { box: top(auto).right(0).bottom(0).left(auto).flexDir(column-reverse) },
      },
    },
  })
>
  <slot></slot>
</column>
