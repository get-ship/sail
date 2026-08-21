<!--
  <sail-citation> — numbered source pill. Click → host onOpen({ source, index }).
  No navigation; consumer decides open target.
-->

<scope>
  props: {
    source = ''
    index = 0
  }

  style pill = { size({ min-h: 0 }).pad({ y: half, l: 1, r: 2 }).radius(full).fg(foreground).bg(muted).border({ color: border, width: 1 }).elevation(none).cursor(pointer).type({ size: xs, weight: normal, leading: snug }), when: { hover: bg(selected-bg).border({ color: primary, width: 1 }) } }
  style badge = { size({ h: 4, min-w: 4 }).pad({ x: 1 }).fg(primary-foreground).bg(primary).radius(full) }
  style badgeNum = { display(contents).type({ weight: semibold }) }
  style label = { type({ wrap: truncate }) }
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
