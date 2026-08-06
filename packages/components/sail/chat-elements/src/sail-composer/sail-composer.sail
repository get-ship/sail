<!--
  <sail-composer> — the message input + send control. Single-line by
  default; `multiline` switches to a growing textarea. Enter submits;
  Shift+Enter inserts a newline (multiline only). Composes a bare
  <button> for the send action.

  Attachments (files, canvases, code snippets) are owned by the
  consumer: render their chips into the `attachments` slot and set the
  `attachments` JS-property on the host. The composer echoes that array
  back in the submit event — it never fabricates attachment data.

  Focus lives on the frame (`focus-within:`), not on the field. The frame
  carries `disabled` as a `data-state` token so the dimmed surface is one
  `state-disabled:` rule on the element that draws it.

  Props
  ─────────────────────────────────────────────────────────────────────
  placeholder  (string)  Input placeholder.
  disabled     (bool)    Disables the input + send button.
  multiline    (bool)    Use a textarea + Shift+Enter for newlines.
  ─────────────────────────────────────────────────────────────────────
  Slots
    attachments  attachment chips (file / canvas / code)
    trailing     extra actions beside Send
  ─────────────────────────────────────────────────────────────────────
  Events
    sail-composer-submit  { detail: { text: string, attachments: unknown[] } }
  ─────────────────────────────────────────────────────────────────────
-->

<scope>
  props: {
    placeholder = ''
    disabled = false
    multiline = false
  }
</scope>

<controller>

import type { Action, ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './sail-composer.sail'

export default controller((_ctx: ControllerScope<Scope>) => {
    const text = signal('')
    // Consumer-owned attachment payloads. The composer never fabricates
    // these — it echoes whatever the host assigned in the submit event.
    const attachments = signal<unknown[]>([])
    // The submit event is dispatched from the host, so the handlers need it.
    // `mount` runs after the template is built, but an action only REGISTERS
    // listeners at attach time — by the time any of them fires, mount has run
    // and this is set.
    let hostEl: Element | null = null

    mount((host) => {
      hostEl = host
      // Expose `attachments` as a JS-property: `el.attachments = [...]`.
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
      hostEl?.dispatchEvent(
        new CustomEvent('sail-composer-submit', {
          detail: { text: trimmed, attachments: [...attachments()] },
          bubbles: true,
          composed: true,
        }),
      )
      text.set('')
    }

    const onKeydown = (e: KeyboardEvent) => {
      if (e.key !== 'Enter') return
      // Shift+Enter inserts a newline in multiline mode; plain Enter
      // always submits. Single-line mode submits on Enter regardless.
      if (_ctx.multiline() && e.shiftKey) return
      e.preventDefault()
      submit()
    }

    // `multiline` toggles the {#if}, swapping <textarea> for <input>. The
    // action re-runs on whichever element mounts and tears down with the one
    // that leaves — the rebind that used to need a ref + tracking effect is
    // just the action's lifecycle.
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

>
  style.pad(2).fontFamily(body).fg(foreground).bg(background).borderWidth(1).border(border).radius(lg).when({ focusWithin: style.borderWidth(1).border(primary).outline(ring), stateDisabled: style.bg(muted).opacity(75) })

  <div>
    style.display(contents)

    <slot name="attachments"></slot>
  </div>
  <!-- nowrap: default row wraps and parks Send under the field in tight tiles. -->
  <row nowrap gap="2" align="center">
    <if {multiline()}>
      <textarea
        rows="1"
        placeholder={placeholder()}
        disabled={disabled()}
        bind:value={text}
        use:fieldKeys
        grow min-w="0"

      >
        style.minW(0).minH(0).padY(1_5).padX(2).fontFamily(body).fontSize(base).leading(normal).fg(foreground).bg(transparent).borderWidth(0).radius(none).shadow(none).overflowY(auto)
</textarea>
    </if>
    <if {!multiline()}>
      <input
        type="text"
        placeholder={placeholder()}
        disabled={disabled()}
        bind:value={text}
        use:fieldKeys
        grow min-w="0"

>
        style.minW(0).minH(0).padY(1_5).padX(2).fontFamily(body).fontSize(base).leading(normal).fg(foreground).bg(transparent).borderWidth(0).radius(none).shadow(none)
      </input>
    </if>
    <row nowrap gap="1" align="center" shrink="0">
      <slot name="trailing"></slot>
      <button
        use:sendClick
        type="button"
        variant="solid"
        tone="primary"
        size="sm"
        shrink="0"
        aria-label="Send"
        disabled={disabled() || text().trim().length === 0}
      >Send</button>
    </row>
  </row>
</column>
