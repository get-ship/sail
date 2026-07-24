<!--
  <avatar> — layout row centers; avatar-media img pack; gradient host sx/CVA.
  Variance = CVA-in-props axes + optional compound only. tone/size/shape are
  ordinary props/axes, not language channels.
-->
<scope>
  props: {
    src = ''
    'icon-slug' = ''
    name = ''
    initials = ''
    tone: {
      // Token PE gradient (primary→accent); not TW gradient utility pack.
      default: 'bg-gradient-primary-accent color: foreground',
      primary: 'bg-none bg: tone color: tone-contrast',
      accent: 'bg-none bg: tone color: tone-contrast',
      info: 'bg-none bg: tone-bg color: tone-fg',
      success: 'bg-none bg: tone-success-bg color: success',
      warning: 'bg-none bg: tone-warning-bg color: warning',
      destructive: 'bg-none bg: tone-bg color: tone-fg',
      muted: 'bg-none bg: tone-bg color: tone-fg',
    }
    size: {
      md: 'size: 7 font-size: base',
      xs: 'size: 5 font-size: xs',
      sm: 'size: control-h-sm font-size: xs',
      lg: 'size: control-h-lg font-size: lg',
      xl: 'size: control-h-xl font-size: 2xl',
    }
    shape: {
      round: 'rounded: full',
      square: 'rounded: sm',
    }
  }
</scope>
<row inline nowrap align="center" justify="center" use:mediaFallback sx="relative shrink: 0 font-family: body font-weight: bold leading: none tracking: wide color: foreground overflow: hidden select: none align: middle size: 7 font-size: base rounded: full shadow: inset-ring bg-gradient-primary-accent avatar-media"></row>
