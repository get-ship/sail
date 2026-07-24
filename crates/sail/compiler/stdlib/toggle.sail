<!--
  <toggle> — first:/last: join + variant/orientation/size CVA. Orientation may be
  structural-stamped by the group; size/variant are author props on each toggle.
-->
<scope>
  props: {
    value = ''
    disabled = false
    variant: {
      outline: 'rounded: none -ms: px first:ms: 0 first:rounded-l: md last:rounded-r: md',
      ghost: 'border: transparent rounded: md ms: 0 me: 1 last:me: 0',
      segmented: 'border: transparent rounded: sm ms: 0 me: 1 last:me: 0',
    }
    orientation: {
      horizontal: '',
      vertical: '',
    }
    size: {
      md: 'font-size: sm h: control-h-md px: 3',
      sm: 'font-size: xs h: control-h-sm px: 2_5',
      lg: 'font-size: base h: control-h-lg px: 4',
    }
  }
  compound: [
    { when: { orientation: 'vertical', variant: 'outline' }, sx: 'rounded: none first:rounded-t: md last:rounded-b: md -mt: px first:mt: 0 ms: 0 me: 0' },
    { when: { variant: 'ghost' }, sx: 'border: transparent rounded: md ms: 0 mt: 0 me: 1 last:me: 0' },
    { when: { variant: 'segmented' }, sx: 'border: transparent rounded: sm ms: 0 mt: 0 me: 1 last:me: 0' },
    { when: { orientation: 'vertical', variant: 'ghost' }, sx: 'border: transparent rounded: md ms: 0 mt: 0 me: 0 mb: 1 last:mb: 0' },
    { when: { orientation: 'vertical', variant: 'segmented' }, sx: 'border: transparent rounded: sm ms: 0 mt: 0 me: 0 mb: 1 last:mb: 0' },
  ]
</scope>
<button type="button" sx="relative font-family: body font-weight: medium leading: none color: foreground bg: background cursor: pointer whitespace: nowrap select: none font-size: sm h: control-h-md px: 3 border border: border rounded: none -ms: px first:ms: 0 first:rounded-l: md last:rounded-r: md tap-highlight: none transition: colors duration: fast ease: out hover:bg: muted pressed:bg: foreground pressed:color: background pressed:border: foreground pressed:z: 1 disabled:opacity: 50 disabled:cursor: not-allowed focus:outline: solid focus:outline: 2 focus:outline: ring focus:z: 2">
  <slot></slot>
</button>
