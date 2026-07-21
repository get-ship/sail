<!--
  <switch> — native checkbox restyled as sliding toggle. No controller.

  Host paint: cursor/align/m-0 on sx. Residue: appearance:none track/thumb
  geometry, size/tone channels, checked/hover/disabled/focus, forced-colors,
  reduced-motion.
-->

<scope>
  props: {
    checked = false
    disabled = false
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    size: 'sm' | 'md' | 'lg' = 'md'
  }
</scope>

<input type="checkbox" role="switch" sx="inline-block align-middle m-0 cursor-pointer" />
