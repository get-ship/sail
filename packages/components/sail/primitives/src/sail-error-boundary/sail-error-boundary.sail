<!--
  <error-boundary> — render-fault isolation. Innermost boundary catches;
  fallback: slot=fallback, else fallback-text, else built-in + Retry.
  host.reset() re-renders the protected subtree.
-->

<scope>
  props: {
    'fallback-text' = ''
  }
</scope>

<controller>
import type { BoundarySource, ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './sail-error-boundary.sail'

interface HostWithReset extends HostBag {
  reset?: () => void
}

export default controller((_ctx: ControllerScope<Scope>, _host?: HostBag) => {
    const caughtError = signal<Error | null>(null)
    const hasFallbackSlot = signal<boolean>(false)
    // Host not in scope factory — publish via signal set in mount.
    const hostEl = signal<HostBag | null>(null)
    const reset = (): void => {
      caughtError.set(null)
      const h = hostEl()
      if (h) hostSetAttr(h, 'data-error-message', null)
    }
    mount((host) => {
      hostEl.set(host as HostBag)
      // Clone inert <template slot="fallback"> into a live slotted wrapper.
      const templates = host.querySelectorAll(':scope > template[slot="fallback"]')
      for (const tpl of Array.from(templates as ArrayLike<HostBag>)) {
        const wrap = hostCreateElement('div') as HostBag
        hostSetAttr(wrap, 'slot', 'fallback')
        const content = (tpl as HostBag & { content?: { cloneNode(d?: boolean): unknown }; replaceWith?(n: unknown): void }).content
        if (content) (wrap as HostBag).appendChild?.(content.cloneNode(true))
        ;(tpl as HostBag & { replaceWith?(n: unknown): void }).replaceWith?.(wrap)
      }
      hasFallbackSlot.set(!!host.querySelector(':scope > [slot="fallback"]'))

      attachBoundary(host, {
        catch(err, source) {
          if (caughtError() !== null) return
          const error = err instanceof Error ? err : new Error(String(err))
          hostSetAttr(host, 'data-error-message', error.message)
          caughtError.set(error)
          ;(host as HostBag).onErrorCaught?.({
            error,
            source: source as BoundarySource,
          })
        },
        reset,
      })

      ;(host as HostWithReset).reset = reset

      return () => {
        detachBoundary(host)
        delete (host as HostWithReset).reset
      }
    })

    return { caughtError, hasFallbackSlot, reset, hostEl }
  })

</controller>
<if {caughtError() === null}>
  <slot></slot>
{else => <if {hasFallbackSlot()}>
    <slot name="fallback"></slot>
  {else => <row
    role="alert"
    gap="3"
    align="start"
    style({ pad({ x: 5, y: 4 }).border({ color: destructive, width: 1 }).radius(md).bg(muted).fg(foreground).elevation(xs) })
  >
    <span style({ display(contents).type({ family: body, size: sm }) })>
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
        style({ size({ w: icon-size-lg, h: icon-size-lg, min-w: icon-size-lg }).margin({ t: 1 }).fg(destructive) })
      >
        <path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z"></path>
        <path d="M12 9v4"></path>
        <path d="M12 17h.01"></path>
      </svg>
      <column gap="1">
        <div style({ fg(muted-foreground) })>
          <span style({ display(contents).type({ leading: normal }) })>
            {fallbackText() || 'Something went wrong'}
          </span>
        </div>
        <button type="button" variant="outline" size="xs" on:click={reset}>Retry</button>
      </column>
    </span>
  </row>}</if>}</if>
