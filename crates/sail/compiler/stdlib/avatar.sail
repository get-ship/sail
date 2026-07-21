<!--
  <avatar> — identity frame (image/icon/initials). use:mediaFallback owns render/fallback.

  Paint: host sx + CVA size/shape/tone maps (first key = PE).
  Contracts: PE gradient fill, flex center, letter-spacing, inset ring, img anatomy.
-->

<scope>
  props: {
    src = ''
    'icon-slug' = ''
    name = ''
    initials = ''
    // first key = default = PE (gradient fill lives in contracts)
    tone: {
      default: 'text-foreground',
      primary: 'bg-tone text-tone-contrast',
      accent: 'bg-tone text-tone-contrast',
      info: 'bg-tone-bg text-tone-fg',
      success: 'bg-tone-success-bg text-success',
      warning: 'bg-tone-warning-bg text-warning',
      destructive: 'bg-tone-bg text-tone-fg',
      muted: 'bg-tone-bg text-tone-fg',
    }
    // first key = default = PE md
    size: {
      md: 'size-7 text-base',
      xs: 'size-5 text-xs',
      sm: 'size-control-h-sm text-xs',
      lg: 'size-control-h-lg text-lg',
      xl: 'size-control-h-xl text-2xl',
    }
    // first key = default = PE round
    shape: {
      round: 'rounded-full',
      square: 'rounded-sm',
    }
  }
</scope>

<span
  use:mediaFallback
  sx="relative shrink-0 font-body font-bold leading-none text-foreground overflow-hidden select-none align-middle size-7 text-base rounded-full"
></span>
