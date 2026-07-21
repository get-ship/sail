<!--
  <avatar> — identity frame (image/icon/initials). use:mediaFallback owns render/fallback.

  Host paint: typeface/overflow/select/align on sx. Residue: size channels,
  gradient default fill, tone matrix, inset ring shadow, shape, img anatomy,
  letter-spacing 0.02em, flex center geometry.
-->

<scope>
  props: {
    src = ''
    'icon-slug' = ''
    name = ''
    initials = ''
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    size: 'xs' | 'sm' | 'md' | 'lg' | 'xl' = 'md'
    shape: 'round' | 'square' = 'round'
  }
</scope>

<span use:mediaFallback sx="font-body font-bold leading-none text-foreground overflow-hidden select-none align-middle"></span>
