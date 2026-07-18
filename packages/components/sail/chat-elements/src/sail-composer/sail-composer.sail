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
import { controller, effect, ref, signal } from '@ship/sail-runtime'
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './sail-composer.sail'

export default controller((_ctx: ControllerScope<Scope>) => {
    const text = signal('')
    // Consumer-owned attachment payloads. The composer never fabricates
    // these — it echoes whatever the host assigned in the submit event.
    const attachments = signal<unknown[]>([])
    const fieldEl = ref<HTMLInputElement | HTMLTextAreaElement>(null)
    const sendEl = ref<HTMLElement>(null)

    mount((host) => {
      // Expose `attachments` as a JS-property: `el.attachments = [...]`.
      const init = (host as unknown as { attachments?: unknown[] }).attachments
      if (init != null) attachments.set(init)
      Object.defineProperty(host, 'attachments', {
        configurable: true,
        get: () => attachments(),
        set: (v: unknown[]) => { attachments.set(v) },
      })

      const submit = () => {
        if (_ctx.disabled()) return
        const trimmed = text().trim()
        if (trimmed.length === 0) return
        host.dispatchEvent(
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

      // `fieldEl` swaps element when `multiline` toggles the {#if}, so
      // track it via effect to rebind keydown onto the live field.
      const stopField = effect(() => {
        // HTMLElement (not the input|textarea union): addEventListener's typed
        // overload resolves the 'keydown' listener cleanly on a single element
        // type, where the union receiver collapses it to a non-assignable shape.
        const field = fieldEl() as HTMLElement | null
        if (!field) return
        field.addEventListener('keydown', onKeydown)
        return () => field.removeEventListener('keydown', onKeydown)
      })

      const stopSend = effect(() => {
        const send = sendEl() as HTMLElement | null
        if (!send) return
        send.addEventListener('click', submit)
        return () => send.removeEventListener('click', submit)
      })

      return () => {
        stopField()
        stopSend()
      }
    })

    return { text, attachments, fieldEl, sendEl }
  })

</controller>
<column
  gap="2"
  data-state={disabled() ? 'disabled' : ''}
  sx="p-2 font-body text-foreground bg-background border border-border rounded-lg focus-within:border focus-within:border-primary focus-within:ring-tone-primary-ring state-disabled:bg-muted state-disabled:opacity-75"
>
  <div sx="contents">
    <slot name="attachments"></slot>
  </div>
  <row gap="2" align="end">
    {#if multiline()}
      <textarea
        rows="1"
        placeholder={placeholder()}
        disabled={disabled()}
        bind:value={text}
        bind:this={fieldEl}
        sx="grow-1 min-w-0 min-h-0 py-1 px-half font-body text-base leading-normal text-foreground bg-transparent border-0 rounded-none shadow-none overflow-y-auto"
      ></textarea>
    {/if}
    {#if !multiline()}
      <input
        type="text"
        placeholder={placeholder()}
        disabled={disabled()}
        bind:value={text}
        bind:this={fieldEl}
        sx="grow-1 min-w-0 min-h-0 py-1 px-half font-body text-base leading-normal text-foreground bg-transparent border-0 rounded-none shadow-none"
      />
    {/if}
    <row gap="1" align="center" sx="grow-0">
      <slot name="trailing"></slot>
      <button
        bind:this={sendEl}
        solid
        primary
        sm
        aria-label="Send"
        disabled={disabled() || text().trim().length === 0}
      >Send</button>
    </row>
  </row>
</column>
