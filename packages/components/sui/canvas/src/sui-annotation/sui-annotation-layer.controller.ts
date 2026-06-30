import { controller, signal, derived, ref } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-annotation-layer.sui'
import { type AnnotationEvent, buildAnnotationEvent } from './sui-annotation.internals'

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

export default controller({
  scope: (props: ViewScope<Scope>, _host?: Element) => {
    const viewIdProp = props.viewId
    const endpointProp = props.endpoint
    const stagingProp = props.staging

    const staged = signal<AnnotationEvent[]>([])
    const pending = signal<Pending | null>(null)
    const drag = signal<Drag | null>(null)
    const noteText = signal('')

    // The host element is not present in the scope factory (the post-
    // convergence "SUI names no host" compile stops passing it as the 2nd
    // arg), so read it through this signal — set in `mount` once the
    // runtime hands over the real element. The `sendOne` / `confirmNote`
    // closures below emit `sui-annotation-{staged,sent,error}` on the host,
    // so they must read the host from this signal, not the dead scope arg.
    const hostEl = signal<HTMLElement | null>(null)

    const _positions: Record<string, { x: number; y: number }> = {}

    const dragVisible = derived(() => {
      const d = drag()
      if (!d) return false
      return Math.abs(d.curX - d.startX) > 4 || Math.abs(d.curY - d.startY) > 4
    })

    const dragRectStyle = derived(() => {
      const d = drag()
      if (!d) return ''
      const x = Math.min(d.startX, d.curX)
      const y = Math.min(d.startY, d.curY)
      const w = Math.abs(d.curX - d.startX)
      const h = Math.abs(d.curY - d.startY)
      return `left:${x}px;top:${y}px;width:${w}px;height:${h}px`
    })

    const popoverStyle = derived(() => {
      const p = pending()
      if (!p) return ''
      const left = Math.min(p.x + 12, 10_000)
      const top = Math.min(p.y + 12, 10_000)
      return `left:${left}px;top:${top}px`
    })

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
        const resp = await fetch(ep, {
          method: 'POST',
          credentials: 'include',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(ev),
        })
        if (!resp.ok) {
          hostEl()?.dispatchEvent(
            new CustomEvent('sui-annotation-error', {
              detail: { code: `http_${resp.status}`, id: ev.id },
              bubbles: true,
              composed: true,
            }),
          )
          return
        }
        staged.set(staged().filter((s) => s.id !== ev.id))
        hostEl()?.dispatchEvent(
          new CustomEvent('sui-annotation-sent', {
            detail: { id: ev.id, status: resp.status },
            bubbles: true,
            composed: true,
          }),
        )
      } catch (e) {
        hostEl()?.dispatchEvent(
          new CustomEvent('sui-annotation-error', {
            detail: { code: 'network_error', message: String(e), id: ev.id },
            bubbles: true,
            composed: true,
          }),
        )
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
      hostEl()?.dispatchEvent(
        new CustomEvent('sui-annotation-staged', {
          detail: ev,
          bubbles: true,
          composed: true,
        }),
      )
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

    const onNoteKeydown = (e: KeyboardEvent): void => {
      if (e.key === 'Escape') {
        cancelPending()
      } else if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
        const ta = e.currentTarget as HTMLTextAreaElement
        confirmNote(ta.value)
      }
    }

    const overlayEl = ref<HTMLDivElement>(null)

    return {
      staged,
      pending,
      drag,
      noteText,
      dragVisible,
      dragRectStyle,
      popoverStyle,
      markers,
      overlayEl,
      confirmNote,
      sendNote,
      cancelPending,
      onNoteKeydown,
      sendOne,
      hostEl,
    }
  },

  mount(state, host) {
    // The runtime hands the real host element here. The scope's `sendOne`
    // and `confirmNote` closures read through `hostEl` (the dead-`host`
    // workaround) — publish the element now that it exists, before the
    // first user gesture can fire.
    state.hostEl = host as HTMLElement

    let _drag: Drag | null = null
    let _raf: number | null = null

    const overlayPos = (e: PointerEvent): { x: number; y: number } => {
      const el = state.$signals.overlayEl() as HTMLElement | null
      if (!el) return { x: 0, y: 0 }
      const r = el.getBoundingClientRect()
      return { x: e.clientX - r.left, y: e.clientY - r.top }
    }

    const onPointerDown = (e: PointerEvent): void => {
      const m = state.$signals.mode() as Mode
      if (m === 'off') return
      const t = e.target as HTMLElement
      if (t.closest('[data-annotation-popover]') || t.closest('sui-annotation-marker')) return
      if (state.$signals.pending()) {
        state.$signals.pending.set(null)
        state.$signals.noteText.set('')
        return
      }
      e.preventDefault()
      const p = overlayPos(e)
      _drag = { startX: p.x, startY: p.y, curX: p.x, curY: p.y }
      state.$signals.drag.set(_drag)
      ;(e.target as Element).setPointerCapture?.(e.pointerId)
    }

    const onPointerMove = (e: PointerEvent): void => {
      if (!_drag) return
      const p = overlayPos(e)
      _drag = { ..._drag, curX: p.x, curY: p.y }
      if (_raf !== null) return
      _raf = requestAnimationFrame(() => {
        _raf = null
        if (_drag) state.$signals.drag.set({ ..._drag })
      })
    }

    const onPointerUp = (e: PointerEvent): void => {
      if (!_drag) return
      const p = overlayPos(e)
      const dx = Math.abs(p.x - _drag.startX)
      const dy = Math.abs(p.y - _drag.startY)
      const m = state.$signals.mode() as Mode
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
      state.$signals.drag.set(null)
      state.$signals.noteText.set('')
      if (next) state.$signals.pending.set(next)
    }

    const overlayEl = state.$signals.overlayEl() as HTMLElement | null
    if (overlayEl) {
      overlayEl.addEventListener('pointerdown', onPointerDown)
      overlayEl.addEventListener('pointermove', onPointerMove)
      overlayEl.addEventListener('pointerup', onPointerUp)
    }

    Object.defineProperty(host, 'staged', {
      configurable: true,
      get: () => state.$signals.staged(),
    })

    type LayerHost = HTMLElement & {
      send: () => Promise<void>
      clear: () => void
    }
    ;(host as LayerHost).send = async () => {
      for (const ev of [...state.$signals.staged()]) await state.sendOne(ev)
    }
    ;(host as LayerHost).clear = () => {
      state.$signals.staged.set([])
    }

    return () => {
      if (_raf !== null) {
        cancelAnimationFrame(_raf)
        _raf = null
      }
      if (overlayEl) {
        overlayEl.removeEventListener('pointerdown', onPointerDown)
        overlayEl.removeEventListener('pointermove', onPointerMove)
        overlayEl.removeEventListener('pointerup', onPointerUp)
      }
    }
  },
})
