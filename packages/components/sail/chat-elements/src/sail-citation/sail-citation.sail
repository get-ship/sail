<!--
  <sail-citation> — a numbered source pill. Clicking it emits
  `sail-citation-open` with `{ source, index }` so the consumer can open
  the reference (panel, new tab, footnote scroll — the pill doesn't
  decide). Pure framework primitive: no navigation, no SDK.

  The pill is a bare <button>, so the framework's control contracts own
  its cursor, focus ring, and hover transition; the sx surface only
  restates what a citation pill is (small, round, muted, tinted on hover).

  Props
  ─────────────────────────────────────────────────────────────────────
  source  (string)  URL or label of the cited source.
  index   (number)  1-based citation number shown in the pill.
  ─────────────────────────────────────────────────────────────────────
  Events
    sail-citation-open  { detail: { source: string, index: number } }
  ─────────────────────────────────────────────────────────────────────
-->

<scope>
  props: {
    source = ''
    index = 0
  }
</scope>

<controller>

import type { Action, ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './sail-citation.sail'

export default controller((_ctx: ControllerScope<Scope>) => {
    // The pill dispatches its own open event. `bubbles: true, composed: true`
    // carries it across the shadow boundary, so the action needs the button,
    // not the host.
    const openOnClick: Action<void> = (node) => {
      const onClick = () => {
        ;(node as any).onOpen?.({ source: _ctx.source(), index: _ctx.index() })
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

>
  style({ box: minH(0).padY(half).padL(0_75).padR(2), paint: radius(full).fg(foreground).bg(muted).borderWidth(1).border(border).shadow(none).cursor(pointer), text: fontSize(xs).leading(snug).fontWeight(normal) }).when(hover, { paint: bg(selected-bg).borderWidth(1).border(primary) })

  <row gap="1" align="center">
    <row
      aria-hidden="true"
      gap="0"
      align="center"
      justify="center"

    >
      style({ box: minW(4).h(4).padX(1), paint: fg(primary-foreground).bg(primary).radius(full) })
      <span>
        style({ box: display(contents), text: fontWeight(semibold) })
        {index()}
      </span>
</row>
    <span>
      style({ text: truncate() })
{source()}</span>
  </row>
</button>
