<!--
  <sail-message> — a single chat bubble. Renders one turn from any
  participant in any modality. Pure declarative primitive: the consumer
  supplies the content via the default slot and any per-message actions
  via the `trailing` slot. No controller — role / content-type / status
  drive presentation entirely.

  role / content-type / status ride the root's `data-state` token list, so
  the bubble reads all three through `group-state-<value>:` sx. The three
  vocabularies are disjoint, so one token list carries them without
  collision.

  Props
  ─────────────────────────────────────────────────────────────────────
  role          (union)  user | assistant | system | tool. Drives
                        bubble alignment + tone.
  content-type  (union)  text | code | drawing | terminal-snapshot |
                        tool-call | citation | image. Drives the inner
                        shape (monospace surface for code/terminal,
                        framed media for drawing/image, etc.).
  timestamp     (string) ISO timestamp; surfaced via the `time`
                        element's datetime + shown in the meta row.
  status        (union)  sending | sent | error. Tints the bubble edge.
  ─────────────────────────────────────────────────────────────────────
  Slots
    (default)   message content
    trailing    per-message actions (copy, retry, …)
  ─────────────────────────────────────────────────────────────────────
-->

<scope>
  props: {
    role: 'assistant' | 'user' | 'system' | 'tool' = 'assistant'
    'content-type': 'text' | 'code' | 'drawing' | 'terminal-snapshot' | 'tool-call' | 'citation' | 'image' = 'text'
    timestamp = ''
    status: 'sent' | 'sending' | 'error' = 'sent'
  }

  const host = style.group().maxW(full).fontFamily(body).fg(foreground)
  const meta = style.padX(half).fontSize(xs).fg(muted-foreground)
  const roleLabel = style.fontWeight(medium)
  const bubble = style.padY(2).padX(3).radius(lg).fontSize(base).leading(normal).bg(card).borderWidth(1).border(border).fg(foreground).when({
    groupStateUser: style.bg(tone-primary-bg).borderWidth(1).border(tone-primary-border),
    groupStateSystem: style.bg(tone-muted-bg).fg(muted-foreground),
    groupStateTool: style.bg(tone-accent-bg).borderWidth(1).border(tone-accent-border),
    groupStateError: style.borderWidth(1).border(tone-destructive-border),
    groupStateSending: style.opacity(75),
    groupStateCode: style.fontFamily(mono).fontSize(sm).whitespace(pre-wrap),
    groupStateToolCall: style.fontFamily(mono).fontSize(sm).whitespace(pre-wrap),
    groupStateTerminalSnapshot: style.fontFamily(mono).fontSize(sm).whitespace(pre-wrap).bg(surface-inverse).fg(background).borderWidth(1).border(border-subtle),
  })
  const trailing = style.display(contents)
</scope>

<column
  gap="1"
  align={role() === 'user' ? 'end' : 'stretch'}
  data-state={role() + ' ' + contentType() + ' ' + status()}
  sx={host}
>
  <row gap="2" align="baseline" sx={meta}>
    <span sx={roleLabel}>{role()}</span>
    {#if timestamp()}
      <time datetime={timestamp()}>{timestamp()}</time>
    {/if}
  </row>

  <div sx={bubble}>
    <slot></slot>
  </div>

  <!-- display:contents so a message with no actions has no trailing box (the
       column's gap never opens under it) and slotted actions become items of
       the message column, following its alignment. -->
  <div sx={trailing}>
    <slot name="trailing"></slot>
  </div>
</column>
