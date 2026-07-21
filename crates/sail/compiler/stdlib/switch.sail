<!--
  <switch> — native checkbox restyled as sliding toggle. No controller.
  Host sx: track paint + before: thumb. channels: --sw-*. CVA tone thumb contrast.
-->
<scope>
  props: {
    checked = false
    disabled = false
    tone: {
      default: 'checked:bg-primary focus:outline-primary checked:before:bg-primary-foreground',
      primary: 'checked:bg-tone focus:outline-tone checked:before:bg-primary-foreground',
      info: 'checked:bg-tone focus:outline-tone checked:before:bg-info-foreground',
      success: 'checked:bg-tone focus:outline-tone checked:before:bg-success-foreground',
      warning: 'checked:bg-tone focus:outline-tone checked:before:bg-warning-foreground',
      destructive: 'checked:bg-tone focus:outline-tone checked:before:bg-destructive-foreground',
      accent: 'checked:bg-tone focus:outline-tone checked:before:bg-accent-foreground',
      muted: 'checked:bg-tone focus:outline-tone checked:before:bg-background',
    }
    size: 'sm' | 'md' | 'lg' = 'md'
  }
  channels: {
    default: { 'sw-w': 'control-h-sm', 'sw-h': 'icon-size-md', 'sw-thumb': 'space-3' }
    size: {
      sm: { 'sw-w': 'control-h-xs', 'sw-h': 'icon-size-sm', 'sw-thumb': 'space-2_5' }
      lg: { 'sw-w': 'space-7', 'sw-h': 'icon-size-lg', 'sw-thumb': 'icon-size-sm' }
    }
  }
</scope>
<input type="checkbox" role="switch" sx="appearance-none relative shrink-0 box-border inline-block align-middle m-0 cursor-pointer bg-none bg-muted border border-border-strong rounded-full w-(--sw-w) h-(--sw-h) tap-highlight-none transition-colors duration-fast ease-out hover:bg-hover-bg checked:bg-primary checked:border-selected-border checked:shadow-xs focus:outline-solid focus:outline-2 focus:outline-primary focus:outline-offset-ring-offset disabled:cursor-not-allowed disabled:bg-muted disabled:border-border disabled:saturate-60 disabled:shadow-none before:content-empty before:absolute before:top-0 before:bottom-0 before:my-auto before:left-px before:size-(--sw-thumb) before:bg-surface-1 before:rounded-full before:shadow-sm before:transition-all before:duration-fast before:ease-out checked:before:left-auto checked:before:right-px checked:before:bg-primary-foreground disabled:before:shadow-xs motion-reduce:before:transition-none forced-colors:border-CanvasText forced-colors:checked:bg-Highlight forced-colors:focus:outline-CanvasText" />
