<!--
  <sail-citation> — numbered source pill. Click → host onOpen({ source, index }).
  No navigation; consumer decides open target.
-->

<scope>
  props: {
    source = ''
    index = 0
  }

  style pill = {
    box: minH(0).padY(half).padL(0_75).padR(2),
    paint: radius(full).fg(foreground).bg(muted).borderWidth(1).border(border).shadow(none).cursor(pointer),
    text: fontSize(xs).leading(snug).fontWeight(normal),
    when: {
      hover: { paint: bg(selected-bg).borderWidth(1).border(primary) },
    },
  }
  style badge = {
    box: minW(4).h(4).padX(1),
    paint: fg(primary-foreground).bg(primary).radius(full),
  }
  style badgeNum = {
    box: display(contents),
    text: fontWeight(semibold),
  }
  style label = { text: truncate() }
</scope>

<controller>

import type { Action, ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './sail-citation.sail'

export default controller((_ctx: ControllerScope<Scope>) => {
    // Click target is the pill button; onOpen is the host moment callback.
    const openOnClick: Action<void> = (node) => {
      const onClick = () => {
        ;(node).onOpen?.({ source: _ctx.source(), index: _ctx.index() })
      }
      node.addEventListener('click', onClick)
      return { destroy: () => node.removeEventListener('click', onClick) }
    }

    return { openOnClick }
  })

</controller>
<button
  type="button"
  use:openOnClick
  aria-label={'Open citation ' + index() + ': ' + source()}
  style(pill)
>
  <row gap="1" align="center">
    <row
      aria-hidden="true"
      gap="0"
      align="center"
      justify="center"
      style(badge)
    >
      <span style(badgeNum)>{index()}</span>
    </row>
    <span style(label)>{source()}</span>
  </row>
</button>
