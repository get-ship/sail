<!--
  <sail-composer> — message input + send. multiline → growing textarea;
  Enter submits, Shift+Enter newline (multiline only). Attachments are
  consumer-owned (slot + host JS property); echoed on submit, never fabricated.
  Events: sail-composer-submit { text, attachments }.
-->

<scope>
  props: {
    placeholder = ''
    disabled = false
    multiline = false
  }

  style frame = {
    box: pad(2),
    paint: fg(foreground).bg(background).borderWidth(1).border(border).radius(lg),
    when: {
      focusWithin: { paint: borderWidth(1).border(primary).outline(ring) },
      stateDisabled: { paint: bg(muted).opacity(75) },
    },
  }
  style body = { box: display(contents), text: fontFamily(body) }
  style field = {
    box: grow(1).minW(0).minH(0).padY(1_5).padX(2),
    paint: fg(foreground).bg(transparent).borderWidth(0).radius(none).shadow(none),
    text: fontFamily(body).fontSize(base).leading(normal),
  }
  style fieldMulti = {
    box: grow(1).minW(0).minH(0).padY(1_5).padX(2).overflowY(auto),
    paint: fg(foreground).bg(transparent).borderWidth(0).radius(none).shadow(none),
    text: fontFamily(body).fontSize(base).leading(normal),
  }
  style actions = { box: shrink(0) }
  style send = { box: shrink(0) }
</scope>

<controller>

import type { Action, ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './sail-composer.sail'

export default controller((_ctx: ControllerScope<Scope>) => {
    const text = signal('')
    // Consumer-owned payloads; composer only echoes them on submit.
    const attachments = signal<unknown[]>([])
    let hostEl: Element | null = null

    mount((host) => {
      hostEl = host
      const init = (host as unknown as { attachments?: unknown[] }).attachments
      if (init != null) attachments.set(init)
      Object.defineProperty(host, 'attachments', {
        configurable: true,
        get: () => attachments(),
        set: (v: unknown[]) => { attachments.set(v) },
      })
    })

    const submit = () => {
      if (_ctx.disabled()) return
      const trimmed = text().trim()
      if (trimmed.length === 0) return
      ;(hostEl as any)?.onSubmit?.({ text: trimmed, attachments: [...attachments()] })
      text.set('')
    }

    const onKeydown = (e: KeyboardEvent) => {
      if (e.key !== 'Enter') return
      if (_ctx.multiline() && e.shiftKey) return
      e.preventDefault()
      submit()
    }

    // multiline swaps textarea/input; action rebinds on whichever mounts.
    const fieldKeys: Action<void> = (node) => {
      node.addEventListener('keydown', onKeydown as EventListener)
      return { destroy: () => node.removeEventListener('keydown', onKeydown as EventListener) }
    }

    const sendClick: Action<void> = (node) => {
      node.addEventListener('click', submit)
      return { destroy: () => node.removeEventListener('click', submit) }
    }

    return { text, attachments, fieldKeys, sendClick }
  })

</controller>
<column
  gap="2"
  data-state={disabled() ? 'disabled' : ''}
  style(frame)
>
  <span style(body)>
    <div style({ box: display(contents) })>
      <slot name="attachments"></slot>
    </div>
    <!-- nowrap keeps Send beside the field in tight tiles. -->
    <row nowrap gap="2" align="center">
      <if {multiline()}>
        <textarea
          rows="1"
          placeholder={placeholder()}
          disabled={disabled()}
          bind:value={text}
          use:fieldKeys
          style(fieldMulti)
        ></textarea>
      </if>
      <if {!multiline()}>
        <input
          type="text"
          placeholder={placeholder()}
          disabled={disabled()}
          bind:value={text}
          use:fieldKeys
          style(field)
        >
      </if>
      <row nowrap gap="1" align="center" style(actions)>
        <slot name="trailing"></slot>
        <button
          use:sendClick
          type="button"
          variant="solid"
          tone="primary"
          size="sm"
          aria-label="Send"
          disabled={disabled() || text().trim().length === 0}
          style(send)
        >Send</button>
      </row>
    </row>
  </span>
</column>
