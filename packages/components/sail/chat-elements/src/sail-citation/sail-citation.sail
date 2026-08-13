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
    box: size({ min-h: 0 }).space({ pad-y: half, pad-l: 0_75, pad-r: 2 }),
    paint: radius(full).ink({ color: foreground }).fill({ color: muted }).border({ color: border, width: 1 }).elevation(none).cursor(pointer),
    text: type({ size: xs, weight: normal, leading: snug }),
    when: {
      hover: { paint: fill({ color: selected-bg }).border({ color: primary, width: 1 }) },
    },
  }
  style badge = {
    box: size({ h: 4, min-w: 4 }).space({ pad-x: 1 }),
    paint: ink({ color: primary-foreground }).fill({ color: primary }).radius(full),
  }
  style badgeNum = {
    box: flow({ display: contents }),
    text: type({ weight: semibold }),
  }
  style label = { text: type({ wrap: truncate }) }
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
