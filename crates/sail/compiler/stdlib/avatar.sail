<!--
  <avatar> — layout row centers; avatar-media img pack; gradient host sx/CVA.
-->
<scope>
  props: {
    src = ''
    'icon-slug' = ''
    name = ''
    initials = ''
    tone: {
      default: 'bg-gradient-to-br from-primary to-accent text-foreground',
      primary: 'bg-none bg-tone text-tone-contrast',
      accent: 'bg-none bg-tone text-tone-contrast',
      info: 'bg-none bg-tone-bg text-tone-fg',
      success: 'bg-none bg-tone-success-bg text-success',
      warning: 'bg-none bg-tone-warning-bg text-warning',
      destructive: 'bg-none bg-tone-bg text-tone-fg',
      muted: 'bg-none bg-tone-bg text-tone-fg',
    }
    size: {
      md: 'size-7 text-base',
      xs: 'size-5 text-xs',
      sm: 'size-control-h-sm text-xs',
      lg: 'size-control-h-lg text-lg',
      xl: 'size-control-h-xl text-2xl',
    }
    shape: {
      round: 'rounded-full',
      square: 'rounded-sm',
    }
  }
</scope>
<row inline nowrap align="center" justify="center" use:mediaFallback sx="relative shrink-0 font-body font-bold leading-none tracking-wide text-foreground overflow-hidden select-none align-middle size-7 text-base rounded-full shadow-inset-ring bg-gradient-to-br from-primary to-accent avatar-media"></row>
