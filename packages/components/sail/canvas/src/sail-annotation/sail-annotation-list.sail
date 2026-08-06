<!--
  <sail-annotation-list> — SAIL port of <ship-annotation-list>.
  Compact list of staged AnnotationEvents with per-item remove controls.

  API surface
  ──────────────────────────────────────────────────────────────────────────────
  JS property:
    staged: AnnotationEvent[]  — set to re-render the list (not attribute-backed;
                                  set the property directly from the parent layer)
  ──────────────────────────────────────────────────────────────────────────────
  Events:
    sail-annotation-remove  { detail: { id: string } }  bubbles, composed
      Fired when the user clicks Remove on an item.
  ──────────────────────────────────────────────────────────────────────────────
  Anatomy: role=list of role=listitem rows; each row ends in a Remove button.
  ──────────────────────────────────────────────────────────────────────────────
  Parity vs <ship-annotation-list>:
    staged property           → staged JS property (same)
    ship-annotation-remove    → sail-annotation-remove (renamed)
    attributeChangedCallback JSON parsing → controller setAttribute interception
-->

<controller>

import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './sail-annotation-list.sail'
import type { AnnotationEvent } from './sail-annotation.internals'

export default controller((_ctx: ControllerScope<Scope>, _host?: Element) => {
    const staged = signal<AnnotationEvent[]>([])
    // The host element is not present in the scope factory (the post-
    // convergence "SAIL names no host" compile stops passing it as the 2nd
    // arg), so read it through this signal — set in `mount` once the
    // runtime hands over the real element. The template wires
    // `on:click={() => emitRemove(a.id)}`, so the emit must read the host
    // from a signal, not the dead scope `host` arg.
    const hostEl = signal<HTMLElement | null>(null)

    const emitRemove = (id: string): void => {
      ;(hostEl() as any)?.onRemove?.({ id })
    }

    mount((host) => {
      // The runtime hands the real host element here. The scope's `emitRemove`
      // closure reads through `hostEl` (the dead-`host` workaround) — publish
      // the element now that it exists, before the first user click.
      hostEl.set(host as HTMLElement)
      // Initial `staged` value: the parent layer sets it as a JS property on
      // this element before mount (the host→list contract). Narrow the access
      // with a guard rather than `as any` — the property is the value type
      // we expect, not an untyped escape hatch.
      const untypedHost = host as unknown as { staged?: unknown }
      if ('staged' in untypedHost) {
        const v = untypedHost.staged
        if (Array.isArray(v)) staged.set(v as AnnotationEvent[])
      }

      // The parent layer mutates `staged` via the host JS property
      // (`list.staged = [...]`); install the accessor so it routes through
      // the signal.
      Object.defineProperty(host, 'staged', {
        configurable: true,
        get: () => staged(),
        set: (v: AnnotationEvent[]) => { staged.set(Array.isArray(v) ? v : []) },
      })
    })

    return { staged, emitRemove, hostEl }
  })

</controller>
<if {staged().length === 0}>
  <p>
    style({ box: padY(1).padX(2), paint: fg(muted-foreground), text: fontFamily(body).fontSize(sm) })
No staged annotations.</p>
{else => <column role="list" gap="1">
    style({ paint: fg(foreground) })
    <span>
      style({ box: display(contents), text: fontFamily(body).fontSize(sm) })
      <for {staged() as a} key={a.id}>
      <row role="listitem" data-id={a.id} gap="2" align="center">
        style({ box: padY(1).padX(2), paint: bg(background).borderWidth(1).border(border).radius(sm) })

        <span>
          style({ paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) })
{a.kind}</span>
        <span title={a.note ?? ''}>
          style({ box: grow(1), text: truncate() })
{a.note ?? '(no note)'}</span>
        <button
          type="button"
          aria-label="Remove annotation"
          on:click={() => emitRemove(a.id)}

        >
          style({ box: minH(0).padX(1_5).padY(half), paint: radius(sm).bg(transparent).borderWidth(1).border(border).fg(muted-foreground).cursor(pointer), text: fontSize(xs) }).when(hover, { paint: fg(destructive).borderWidth(1).border(destructive) })
Remove</button>
      </row>
    </for>
    </span>
</column>}</if>
