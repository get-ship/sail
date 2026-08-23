<!--
  <sail-annotation-layer> — capture overlay (point/region) + note popover.
  POST immediately or stage for host send()/clear(). Events: sail-annotation-staged/sent/error.
-->

<scope>
  props: {
    'view-id' = ''
    mode: 'off' | 'point' | 'region' = 'off'
    endpoint = '/api/feedback'
    staging = false
  }
</scope>
<controller>

import type { Action, ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './sail-annotation-layer.sail'
import { type AnnotationEvent, buildAnnotationEvent, postAnnotationEvent } from './sail-annotation.internals'

type Mode = 'off' | 'point' | 'region'

interface Pending {
  kind: 'point' | 'region'
  x: number
  y: number
  selector?: string
  rect?: [number, number, number, number]
}

interface Drag {
  startX: number
  startY: number
  curX: number
  curY: number
}

const DEFAULT_ENDPOINT = '/api/feedback'

export default controller((props: ControllerScope<Scope>, _host?: HostBag) => {
    const viewIdProp = props.viewId
    const endpointProp = props.endpoint
    const stagingProp = props.staging

    const staged = signal<AnnotationEvent[]>([])
    const pending = signal<Pending | null>(null)
    const drag = signal<Drag | null>(null)
    const noteText = signal('')

    // Host is not a setup arg — capture in mount via this signal once the
    // runtime hands over the real element. The `sendOne` / `confirmNote`
    // closures below emit `sail-annotation-{staged,sent,error}` on the host,
    // so they must read the host from this signal, not the dead scope arg.
    const hostEl = signal<HostBag | null>(null)

    const _positions: Record<string, { x: number; y: number }> = {}

    const dragVisible = derived(() => {
      const d = drag()
      if (!d) return false
      return Math.abs(d.curX - d.startX) > 4 || Math.abs(d.curY - d.startY) > 4
    })

    // Overlay-px geometry of the live drag; the template binds it onto the
    // rubber-band rect through the style: channel (runtime geometry).
    // `dragVisible` gates the rect, so the resting value is a zero box.
    const dragRect = derived(() => {
      const d = drag()
      if (!d) return { x: 0, y: 0, w: 0, h: 0 }
      return {
        x: Math.min(d.startX, d.curX),
        y: Math.min(d.startY, d.curY),
        w: Math.abs(d.curX - d.startX),
        h: Math.abs(d.curY - d.startY),
      }
    })

    // The note popover sits just below-right of the captured point, clamped so
    // a wild coordinate can't fling it out of the document. `pending` gates it.
    const popoverPos = derived(() => {
      const p = pending()
      if (!p) return { left: 0, top: 0 }
      return { left: Math.min(p.x + 12, 10_000), top: Math.min(p.y + 12, 10_000) }
    })


    // `off` leaves the overlay inert; point/region arm it for capture.
    const captureState = derived(() => (props.mode() === 'off' ? '' : 'capture'))

    const markers = derived(() =>
      staged().map((a, i) => ({
        id: a.id,
        kind: a.kind,
        index: i,
        rect: a.kind === 'region' && a.target.rect ? a.target.rect.join(',') : '',
        pos: _positions[a.id] ?? { x: 0, y: 0 },
      })),
    )

    async function sendOne(ev: AnnotationEvent): Promise<void> {
      const ep = endpointProp() || DEFAULT_ENDPOINT
      try {
        const resp = await postAnnotationEvent(ep, ev)
        if (!resp.ok) {
          ;(hostEl() as HostBag)?.onError?.({ code: `http_${resp.status}`, id: ev.id })
          return
        }
        staged.set(staged().filter((s) => s.id !== ev.id))
        ;(hostEl() as HostBag)?.onSent?.({ id: ev.id, status: resp.status })
      } catch (e) {
        ;(hostEl() as HostBag)?.onError?.({
          code: 'network_error',
          message: String(e),
          id: ev.id,
        })
      }
    }

    function confirmNote(note: string): void {
      const p = pending()
      if (!p) return
      const trimmed = note.trim()
      if (!trimmed) {
        pending.set(null)
        noteText.set('')
        return
      }
      const ev = buildAnnotationEvent({
        viewId: viewIdProp() || undefined,
        kind: p.kind,
        selector: p.kind === 'point' ? p.selector : undefined,
        rect: p.kind === 'region' ? p.rect : undefined,
        note: trimmed,
      })
      _positions[ev.id] = { x: p.x, y: p.y }
      staged.set([...staged(), ev])
      ;(hostEl() as HostBag)?.onStaged?.(ev)
      pending.set(null)
      noteText.set('')
      if (!stagingProp()) void sendOne(ev)
    }

    const sendNote = (): void => {
      confirmNote(noteText())
    }

    const cancelPending = (): void => {
      pending.set(null)
      noteText.set('')
    }

    const onNoteKeydown = (e: DomEv): void => {
      if (e.key === 'Escape') {
        cancelPending()
      } else if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
        const ta = e.currentTarget as HostBag
        confirmNote(ta.value)
      }
    }

    const overlayAction: Action<void> = (node) => {
      let _drag: Drag | null = null

      const overlayPos = (e: DomEv): { x: number; y: number } => {
        const r = node.getBoundingClientRect()
        return { x: e.clientX - r.left, y: e.clientY - r.top }
      }

      const onPointerDown = (e: DomEv): void => {
        const m = props.mode() as Mode
        if (m === 'off') return
        const t = e.target as HostBag
        if (t.closest('[data-annotation-popover]') || t.closest('sail-annotation-marker')) return
        if (pending()) {
          pending.set(null)
          noteText.set('')
          return
        }
        e.preventDefault()
        const p = overlayPos(e)
        _drag = { startX: p.x, startY: p.y, curX: p.x, curY: p.y }
        drag.set(_drag)
        t.setPointerCapture?.(e.pointerId)
      }

      const onPointerMove = (e: DomEv): void => {
        if (!_drag) return
        const p = overlayPos(e)
        _drag = { ..._drag, curX: p.x, curY: p.y }
        drag.set({ ..._drag })
      }

      const onPointerUp = (e: DomEv): void => {
        if (!_drag) return
        const p = overlayPos(e)
        const dx = Math.abs(p.x - _drag.startX)
        const dy = Math.abs(p.y - _drag.startY)
        const m = props.mode() as Mode
        let next: Pending | null = null
        if (m === 'point' || (m === 'region' && dx < 8 && dy < 8)) {
          next = { kind: 'point', x: p.x, y: p.y, selector: 'body' }
        } else if (m === 'region') {
          const x = Math.min(_drag.startX, p.x)
          const y = Math.min(_drag.startY, p.y)
          const rect: [number, number, number, number] = [
            Math.round(x),
            Math.round(y),
            Math.round(dx),
            Math.round(dy),
          ]
          next = { kind: 'region', x: x + dx / 2, y: y + dy / 2, rect }
        }
        _drag = null
        drag.set(null)
        noteText.set('')
        if (next) pending.set(next)
      }

      node.addEventListener('pointerdown', onPointerDown)
      node.addEventListener('pointermove', onPointerMove)
      node.addEventListener('pointerup', onPointerUp)

      return {
        destroy: () => {
          node.removeEventListener('pointerdown', onPointerDown)
          node.removeEventListener('pointermove', onPointerMove)
          node.removeEventListener('pointerup', onPointerUp)
        },
      }
    }

    mount((host) => {
      // The runtime hands the real host element here. The scope's `sendOne`
      // and `confirmNote` closures read through `hostEl` (the dead-`host`
      // workaround) — publish the element now that it exists, before the
      // first user gesture can fire.
      hostEl.set(host)

      defineHostProperty(asSailHost(host) as HostBag, 'staged', {get: () => staged(),})

      type LayerHost = HostBag & {
        send: () => Promise<void>
        clear: () => void
      }
      ;(host as LayerHost).send = async () => {
        for (const ev of [...staged()]) await sendOne(ev)
      }
      ;(host as LayerHost).clear = () => {
        staged.set([])
      }
    })

    return {
      staged,
      pending,
      drag,
      noteText,
      dragVisible,
      dragRect,
      popoverPos,
      captureState,
      markers,
      overlayAction,
      confirmNote,
      sendNote,
      cancelPending,
      onNoteKeydown,
      sendOne,
      hostEl,
    }
  })

</controller>
<div style({ position(relative).size({ w: 100%, h: 100% }) })>  <span style({ display(contents).type({ family: body }) })>    <slot></slot>
  <div use:overlayAction data-state={captureState()} style({ position({ scheme: absolute, t: 0, b: 0, l: 0, r: 0 }).position({ z: dropdown }).pointerEvents(none), when: { stateCapture: pointerEvents(auto) } })>
    <if {dragVisible()}>
      <div
        aria-hidden="true"
        style({ position({ scheme: absolute, t: bind(dragRect.y), l: bind(dragRect.x) }).size({ w: bind(dragRect.w), h: bind(dragRect.h) }).pointerEvents(none).bg(hover-bg).border({ color: primary, width: 1, style: dashed }).radius(sm) })></div>
    </if>
    <for {markers() as m} key={m.id}>
      <sail-annotation-marker
        kind={m.kind}
        index={m.index}
        rect={m.rect}
        x={m.pos.x}
        y={m.pos.y}
      ></sail-annotation-marker>
    </for>
    <if {pending()}>
      <column
        data-annotation-popover
        gap="2"
                on:pointerdown={(e) => e.stopPropagation()}

       style({ position({ scheme: absolute, t: bind(popoverPos.top), l: bind(popoverPos.left) }).size({ min-w: panel-max-w-sm }).pad(2).position({ z: sticky }).bg(surface).fg(foreground).border({ color: border, width: 1 }).radius(md).elevation(lg).pointerEvents(auto) })>

        <textarea placeholder="Add a note…" bind:value={noteText} on:keydown={onNoteKeydown} style({ size({ w: 100% }).radius(sm) }) />
        <row gap="2" justify="end">
          <button type="button" on:click={cancelPending} style({ pad({ x: 2_5 }).radius(sm).cursor(pointer).type({ size: sm }) })>Cancel</button>
          <button type="button" data-primary variant="solid" tone="primary" on:click={sendNote} style({ pad({ x: 2_5 }).radius(sm).cursor(pointer).type({ size: sm }) })>Send</button>
        </row>
      </column>
    </if>
  </div>
  </span>
</div>
