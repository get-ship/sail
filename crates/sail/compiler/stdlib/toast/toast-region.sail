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

  style base = {position({ scheme: fixed, t: 0, r: auto, b: auto, l: 0, z: toast }).pad(4).size({ w: 24rem, max-w: 100% }).pointerEvents(none)}
</scope>

<column
  gap="2"
  label="Notifications"
  style(base, {match: {position: {top-start: position({ t: 0, r: auto, b: auto, l: 0 }), top-end: position({ t: 0, r: 0, b: auto, l: auto }), bottom-start: position({ t: auto, r: auto, b: 0, l: 0 }).dir(column-reverse), bottom-end: position({ t: auto, r: 0, b: 0, l: auto }).dir(column-reverse)}}})
>
  <slot></slot>
</column>
