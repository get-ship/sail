<!--
  <spinner> — channels: --bw; .ring child sx for two-tone border + animate-spin.
-->
<scope>
  props: {
    size: {
      md: 'size-icon-size-md',
      xs: 'size-icon-size-2xs',
      sm: 'size-icon-size-sm',
      lg: 'size-icon-size-xl',
    }
    tone: {
      default: 'text-muted-foreground',
      primary: 'text-tone-fg',
      info: 'text-tone-fg',
      success: 'text-tone-fg',
      warning: 'text-tone-fg',
      destructive: 'text-tone-fg',
      accent: 'text-tone-fg',
    }
    centered = false
  }
  compound: [
    { when: { centered: true }, sx: 'block my-8 mx-auto' },
  ]
  channels: {
    default: { bw: 'control-stroke' }
    size: {
      xs: { bw: '1.5px' }
      sm: { bw: '1.5px' }
      lg: { bw: '2.5px' }
    }
  }
</scope>
<span class="spin" role="status" aria-label="Loading" sx="inline-block align-middle size-icon-size-md text-muted-foreground">
  <span class="ring" aria-hidden="true" sx="block size-full border-(--bw) border-solid border-border-subtle border-r-currentColor rounded-full animate-spin"></span>
</span>
