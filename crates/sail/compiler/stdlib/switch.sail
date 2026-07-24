<!--
  <switch> — native checkbox restyled as sliding toggle. No controller.
  Host sx: track paint + before: thumb PE md; size CVA overrides track/thumb.
  CVA tone thumb contrast.
  Variance = CVA-in-props axes + optional compound only. tone/size are ordinary
  props/axes, not language channels.
  Paint: rest track/border/thumb from component.switch tokens
         (cssNames switch, switch-border, switch-thumb); checked/tone stay global/tone.
-->
<scope>
  props: {
    checked = false
    disabled = false
    tone: {
      default: 'checked:bg: primary focus:outline: primary checked:before:bg: primary-foreground',
      primary: 'checked:bg: tone focus:outline: tone checked:before:bg: primary-foreground',
      info: 'checked:bg: tone focus:outline: tone checked:before:bg: info-foreground',
      success: 'checked:bg: tone focus:outline: tone checked:before:bg: success-foreground',
      warning: 'checked:bg: tone focus:outline: tone checked:before:bg: warning-foreground',
      destructive: 'checked:bg: tone focus:outline: tone checked:before:bg: destructive-foreground',
      accent: 'checked:bg: tone focus:outline: tone checked:before:bg: accent-foreground',
      muted: 'checked:bg: tone focus:outline: tone checked:before:bg: background',
    }
    size: {
      md: 'w: control-h-sm h: icon-size-md before:size: 3',
      sm: 'w: control-h-xs h: icon-size-sm before:size: 2_5',
      lg: 'w: 7 h: icon-size-lg before:size: icon-size-sm',
    }
  }
</scope>
<input type="checkbox" role="switch" sx="appearance: none relative shrink: 0 box: border inline-block align: middle m: 0 cursor: pointer bg: none bg: switch border border: switch-border rounded: full w: control-h-sm h: icon-size-md before:size: 3 tap-highlight: none transition: colors duration: fast ease: out hover:bg: hover-bg checked:bg: primary checked:border: selected-border checked:shadow: xs focus:outline: solid focus:outline: 2 focus:outline: primary focus:outline-offset: ring-offset disabled:cursor: not-allowed disabled:bg: switch disabled:border: border disabled:saturate: 60 disabled:shadow: none before:content: empty before:absolute before:top: 0 before:bottom: 0 before:my: auto before:left: px before:bg: switch-thumb before:rounded: full before:shadow: sm before:transition: all before:duration: fast before:ease: out checked:before:left: auto checked:before:right: px checked:before:bg: primary-foreground disabled:before:shadow: xs motion-reduce:before:transition: none forced-colors:border: CanvasText forced-colors:checked:bg: Highlight forced-colors:focus:outline: CanvasText" />
