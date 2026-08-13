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

  style base = {box: place({ position: fixed, top: 0, right: auto, bottom: auto, left: 0, z: toast }).space({ pad: 4 }).size({ w: panel-w-toast }), paint: pointerEvents(none)}
</scope>

<column
  gap="2"
  style(base, {match: {position: {top-start: {box: place({ top: 0, right: auto, bottom: auto, left: 0 })}, top-end: {box: place({ top: 0, right: 0, bottom: auto, left: auto })}, bottom-start: {box: place({ top: auto, right: auto, bottom: 0, left: 0 }).flow({ dir: column-reverse })}, bottom-end: {box: place({ top: auto, right: 0, bottom: 0, left: auto }).flow({ dir: column-reverse })}}}})
>
  <slot></slot>
</column>
