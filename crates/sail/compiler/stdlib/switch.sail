<!--
  <switch> — native checkbox restyled as sliding toggle. No controller.

  Host: cursor/align/m-0 on sx. Track/thumb anatomy (appearance:none, ::before,
  size channels, checked/hover/disabled/focus, forced-colors, reduced-motion)
  and PE tone defaults under :not([tone]) live in contracts.
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
