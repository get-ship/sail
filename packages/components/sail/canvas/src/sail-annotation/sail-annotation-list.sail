<!--
  <sail-annotation-list> — compact list of staged annotations with Remove.
  staged is a host JS property (array identity), not a Scope attr.
  Remove → host onRemove({ id }) (sail-annotation-remove consumers).
-->

<scope>
  style empty = { pad({ x: 2, y: 1 }).fg(muted-foreground).type({ family: body, size: sm }) }
  style list = { fg(foreground).type({ family: body, size: sm }) }
  style row = { pad({ x: 2, y: 1 }).bg(background).border({ color: border, width: 1 }).radius(sm) }
  style kind = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
  style note = { grow(1).type({ wrap: truncate }) }
  style remove = { size({ min-h: 0 }).pad({ x: 1_5, y: half }).radius(sm).bg(transparent).border({ color: border, width: 1 }).fg(muted-foreground).cursor(pointer).type({ size: xs }), when: { hover: fg(destructive).border({ color: destructive, width: 1 }) } }
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
