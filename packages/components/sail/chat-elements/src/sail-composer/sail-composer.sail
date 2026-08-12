<!--
  <sail-composer> — chat message input + send.

  Product default is a multi-line growing field (not a one-line form input).
  Empty state still shows ~3 lines of room so the control reads as a composer.
  multiline={false} opts into a single-line field.
  Enter submits; Shift+Enter newline (multiline only).
  Callback: onSubmit({ text, attachments }) / on-submit= in templates.
-->

<scope>
  props: {
    placeholder = ''
    disabled = false
    multiline = true
  }

  style frame = {
    box: w(full).minW(0).pad(3),
    paint: fg(foreground).bg(background).borderWidth(1).border(border).radius(lg),
    when: {
      focusWithin: { paint: borderWidth(1).border(primary).outline(ring) },
      stateDisabled: { paint: bg(muted).opacity(75) },
    },
  }
  // Chat field: full width. rows="3" + fixed autosize (never height:0) is the
  // empty-state floor — L1 size=md minH(control-h-md) only floors, it does not
  // cap. maxH caps the open box once content grows.
  style fieldMulti = {
    box: w(full).minW(0).minH(20).maxH(48).padY(2).padX(2).overflowY(auto),
    paint: fg(foreground).bg(transparent).borderWidth(0).radius(none).shadow(none),
    text: fontFamily(body).fontSize(base).leading(relaxed),
  }
  style field = {
    box: w(full).minW(0).minH(control-h-md).padY(1_5).padX(2),
    paint: fg(foreground).bg(transparent).borderWidth(0).radius(none).shadow(none),
    text: fontFamily(body).fontSize(base).leading(normal),
  }
  style actions = { box: w(full) }
  style send = { box: shrink(0) }
</scope>

<controller>

import type { Action, ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './sail-composer.sail'

export default controller((_ctx: ControllerScope<Scope>) => {
    const text = signal('')
    const attachments = signal<unknown[]>([])
    let hostEl: HostBag | null = null

    mount((host) => {
      hostEl = host
      const init = (host as { attachments?: unknown[] }).attachments
      if (init != null) attachments.set(init)
      defineHostProperty(asSailHost(host) as HostBag, 'attachments', {
        get: () => attachments(),
        set: (v: unknown[]) => { attachments.set(v) },
      })
    })

    const submit = () => {
      if (_ctx.disabled()) return
      const trimmed = text().trim()
      if (trimmed.length === 0) return
      ;(hostEl)?.onSubmit?.({ text: trimmed, attachments: [...attachments()] })
      text.set('')
    }

    const onKeydown = (e: DomEv) => {
      if (e.key !== 'Enter') return
      if (_ctx.multiline() && e.shiftKey) return
      e.preventDefault()
      submit()
    }

    const fieldKeys: Action<void> = (node) => {
      node.add('keydown', onKeydown)
      return { destroy: () => node.remove('keydown', onKeydown) }
    }

    const sendClick: Action<void> = (node) => {
      node.add('click', submit)
      return { destroy: () => node.remove('click', submit) }
    }

    return { text, attachments, fieldKeys, sendClick }
  })

</controller>
<column
  gap="2"
  data-state={disabled() ? 'disabled' : ''}
  style(frame)
>
  <div style({ box: display(contents) })>
    <slot name="attachments"></slot>
  </div>
  <if {multiline()}>
    <textarea
      rows="3"
      placeholder={placeholder()}
      disabled={disabled()}
      bind:value={text}
      use:fieldKeys
      use:autosize
      style(fieldMulti)
    />
    <row nowrap gap="1" align="center" justify="end" style(actions)>
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
  </if>
  <if {!multiline()}>
    <row nowrap gap="2" align="center" style({ box: w(full) })>
      <input
        type="text"
        placeholder={placeholder()}
        disabled={disabled()}
        bind:value={text}
        use:fieldKeys
        style(field)
      >
      <row nowrap gap="1" align="center" style({ box: shrink(0) })>
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
  </if>
</column>
