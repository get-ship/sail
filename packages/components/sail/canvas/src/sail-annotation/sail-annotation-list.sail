<!--
  <sail-annotation-list> — compact list of staged annotations with Remove.
  staged is a host JS property (array identity), not a Scope attr.
  Remove → host onRemove({ id }) (sail-annotation-remove consumers).
-->

<scope>
  style empty = {
    box: padY(1).padX(2),
    paint: fg(muted-foreground),
    text: fontFamily(body).fontSize(sm),
  }
  style list = {
    paint: fg(foreground),
    text: fontFamily(body).fontSize(sm),
  }
  style row = {
    box: padY(1).padX(2),
    paint: bg(background).borderWidth(1).border(border).radius(sm),
  }
  style kind = {
    paint: fg(muted-foreground),
    text: fontSize(xs).textCase(uppercase).tracking(wide),
  }
  style note = {
    box: grow(1),
    text: truncate(),
  }
  style remove = {
    box: minH(control-h-xs).padX(1_5).padY(half),
    paint: radius(sm).bg(transparent).borderWidth(1).border(border).fg(muted-foreground).cursor(pointer),
    text: fontSize(xs),
    when: {
      hover: { paint: fg(destructive).borderWidth(1).border(destructive) },
    },
  }
</scope>

<controller>

import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './sail-annotation-list.sail'
import type { AnnotationEvent } from './sail-annotation.internals'

export default controller((_ctx: ControllerScope<Scope>, _host?: HostBag) => {
    const staged = signal<AnnotationEvent[]>([])
    // Host is published in mount (scope factory has no host after convergence).
    const hostEl = signal<HostBag | null>(null)

    const emitRemove = (id: string): void => {
      ;(hostEl() as HostBag)?.onRemove?.({ id })
    }

    mount((host) => {
      hostEl.set(host)
      const untypedHost = host as { staged?: unknown }
      if ('staged' in untypedHost) {
        const v = untypedHost.staged
        if (Array.isArray(v)) staged.set(v as AnnotationEvent[])
      }
      // Parent layer assigns `list.staged = [...]` — route through the signal.
      defineHostProperty(asSailHost(host) as HostBag, 'staged', {
        get: () => staged(),
        set: (v: AnnotationEvent[]) => { staged.set(Array.isArray(v) ? v : []) },
      })
    })

    return { staged, emitRemove, hostEl }
  })

</controller>
<if {staged().length === 0}>
  <p style(empty)>No staged annotations.</p>
{else =>
  <column role="list" gap="1" style(list)>
    <for {staged() as a} key={a.id}>
      <row
        role="listitem"
        data-id={a.id}
        gap="2"
        align="center"
        style(row)
      >
        <span style(kind)>{a.kind}</span>
        <span title={a.note ?? ''} style(note)>{a.note ?? '(no note)'}</span>
        <button
          type="button"
          aria-label="Remove annotation"
          on:click={() => emitRemove(a.id)}
          style(remove)
        >
          Remove
        </button>
      </row>
    </for>
  </column>
}</if>
