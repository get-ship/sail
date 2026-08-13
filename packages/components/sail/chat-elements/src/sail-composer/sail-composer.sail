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
    box: size({ w: full, min-w: 0 }).space({ pad: 3 }),
    paint: ink({ color: foreground }).fill({ color: background }).border({ color: border, width: 1 }).radius(lg),
    when: {
      focusWithin: { paint: border({ color: primary, width: 1 }).border({ kind: outline, color: ring }) },
      stateDisabled: { paint: fill({ color: muted }).filter({ opacity: 75 }) },
    },
  }
  // Chat field: full width. rows="3" + fixed autosize (never height:0) is the
  // empty-state floor — L1 size=md minH(control-h-md) only floors, it does not
  // cap. maxH caps the open box once content grows.
  style fieldMulti = {
    box: size({ w: full, min-w: 0, min-h: 20, max-h: 48 }).space({ pad-x: 2, pad-y: 2 }).overflow({ y: auto }),
    paint: ink({ color: foreground }).fill({ color: transparent }).border({ width: 0 }).radius(none).elevation(none),
    text: type({ family: body, size: base, leading: relaxed }),
  }
  style field = {
    box: size({ w: full, min-w: 0, min-h: control-h-md }).space({ pad-x: 2, pad-y: 1_5 }),
    paint: ink({ color: foreground }).fill({ color: transparent }).border({ width: 0 }).radius(none).elevation(none),
    text: type({ family: body, size: base, leading: normal }),
  }
  style actions = { box: size({ w: full }) }
  style send = { box: flow({ shrink: 0 }) }
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
  <div style({ box: flow({ display: contents }) })>
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
    <row nowrap gap="2" align="center" style({ box: size({ w: full }) })>
      <input
        type="text"
        placeholder={placeholder()}
        disabled={disabled()}
        bind:value={text}
        use:fieldKeys
        style(field)
      >
      <row nowrap gap="1" align="center" style({ box: flow({ shrink: 0 }) })>
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
