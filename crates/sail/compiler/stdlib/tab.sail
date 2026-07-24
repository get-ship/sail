<!--
  <tab> — paint + variant/size CVA for pill/enclosed and pad scale (author attrs).
  Paint: idle/selected from component.tab tokens
         (cssNames tab-foreground, tab-selected-foreground, tab-selected).
-->
<scope>
  props: {
    name = ''
    disabled = false
    variant: {
      line: '',
      pill: 'border-b: 0 rounded: sm selected:color: tab-selected-foreground selected:bg: tab-selected selected:shadow: xs selected:border-b: transparent',
      enclosed: 'border border: transparent border-b: 0 rounded-t: md selected:color: tab-selected-foreground selected:border: border selected:bg: tab-selected selected:-mb: px',
    }
    size: {
      md: 'font-size: sm py: 2 px: 3',
      sm: 'font-size: xs py: 1_5 px: 2_5',
      lg: 'font-size: base py: 2_5 px: 4',
    }
  }
</scope>
<button type="button" sx="font-family: body font-weight: medium leading: none color: tab-foreground bg: transparent cursor: pointer whitespace: nowrap select: none font-size: sm py: 2 px: 3 border: 0 border-b: 2 border-b: transparent rounded-t: sm tap-highlight: none transition: colors duration: fast ease: out hover:color: foreground selected:color: tab-selected-foreground selected:font-weight: semibold selected:border-b: tone focus:outline: solid focus:outline: ring-width focus:outline: tone-ring focus:outline-offset: ring-inset disabled:opacity: 50 disabled:cursor: not-allowed">
  <slot></slot>
</button>
