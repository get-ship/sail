<!--
  <spinner> — size/tone CVA on host; .ring child sx for two-tone border + animate-spin.
  Variance = CVA-in-props axes + optional compound only. tone/size are ordinary
  props/axes, not language channels.
-->
<scope>
  props: {
    size: {
      md: 'size: icon-size-md',
      xs: 'size: icon-size-2xs',
      sm: 'size: icon-size-sm',
      lg: 'size: icon-size-xl',
    }
    tone: {
      default: 'color: spinner-foreground',
      primary: 'color: tone-fg',
      info: 'color: tone-fg',
      success: 'color: tone-fg',
      warning: 'color: tone-fg',
      destructive: 'color: tone-fg',
      accent: 'color: tone-fg',
    }
    centered = false
  }
  compound: [
    { when: { centered: true }, sx: 'block my: 8 mx: auto' },
  ]
</scope>
<span class="spin" role="status" aria-label="Loading" sx="inline-block align: middle size: icon-size-md color: spinner-foreground">
  <span class="ring" aria-hidden="true" sx="block size: full border: (--control-stroke) border: solid border: spinner-track border-r: currentColor rounded: full animate-spin"></span>
</span>
