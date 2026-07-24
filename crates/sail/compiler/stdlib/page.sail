<!--
  <page> — size CVA owns cell geometry (stamped from pagination).

  Paint: fg / hover / current from component.page tokens (cssNames
  page-foreground, page-hover-bg, page-current-bg, page-current-foreground).
-->
<scope>
  props: {
    value = ''
    disabled = false
    size: {
      md: 'font-size: sm min-w: control-h-md h: control-h-md',
      sm: 'font-size: xs min-w: control-h-sm h: control-h-sm',
      lg: 'font-size: base min-w: control-h-lg h: control-h-lg',
    }
  }
</scope>
<button type="button" sx="font-family: body font-weight: medium font-size: sm color: page-foreground bg: transparent rounded: md cursor: pointer select: none min-w: control-h-md h: control-h-md px: 2 border border: transparent transition: colors duration: fast ease: out hover:bg: page-hover-bg current:bg: page-current-bg current:color: page-current-foreground current:border: page-current-bg disabled:opacity: 50 disabled:cursor: not-allowed focus:outline: solid focus:outline: 2 focus:outline: ring">
  <slot></slot>
</button>
