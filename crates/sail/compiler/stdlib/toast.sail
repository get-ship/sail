<!--
  <toast> — L1 toast: auto-dismiss + close control via stacked use: behaviors
    (use:autoDismiss, use:dismissControl). tone + duration attrs; PE role/aria-live.

  Host paint: typeface/color/bg/radius/shadow on sx. Flex geometry + 3px tone
  stripe border residue + close-button anatomy stay author CSS.
-->

<scope>
  props: {
    tone = 'default'
    duration = ''
  }
</scope>

<div use:autoDismiss use:dismissControl role="status" aria-live="polite" sx="font-body text-sm text-foreground bg-background rounded-md shadow-md"><slot></slot></div>
