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

  const host = style({ box: maxW(full), paint: group().fg(foreground), text: fontFamily(body) })
  const meta = style({ box: padX(half), paint: fg(muted-foreground), text: fontSize(xs) })
  const roleLabel = style({ text: fontWeight(medium) })
  const bubble = style({ box: padY(2).padX(3), paint: radius(lg).bg(card).borderWidth(1).border(border).fg(foreground), text: fontSize(base).leading(normal) }).when(groupStateUser, { paint: bg(hover-bg).borderWidth(1).border(primary) }).when(groupStateSystem, { paint: bg(muted).fg(muted-foreground) }).when(groupStateTool, { paint: bg(hover-bg).borderWidth(1).border(accent) }).when(groupStateError, { paint: borderWidth(1).border(destructive) }).when(groupStateSending, { paint: opacity(75) }).when(groupStateCode, { text: fontFamily(mono).fontSize(sm).whitespace(pre-wrap) }).when(groupStateToolCall, { text: fontFamily(mono).fontSize(sm).whitespace(pre-wrap) }).when(groupStateTerminalSnapshot, { paint: bg(surface-inverse).fg(background).borderWidth(1).border(border-subtle), text: fontFamily(mono).fontSize(sm).whitespace(pre-wrap) })
  const trailing = style({ box: display(contents) })
</scope>

<column
  gap="1"
  align={role() === 'user' ? 'end' : 'stretch'}
  data-state={role() + ' ' + contentType() + ' ' + status()}

>
  style(host)

  <row gap="2" align="baseline">
    style(meta)

    <span>
      style(roleLabel)
{role()}</span>
    <if {timestamp()}>
      <time datetime={timestamp()}>{timestamp()}</time>
    </if>
  </row>

  <div>
    style(bubble)

    <slot></slot>
  </div>

  <!-- display:contents so a message with no actions has no trailing box (the
       column's gap never opens under it) and slotted actions become items of
       the message column, following its alignment. -->
  <div>
    style(trailing)

    <slot name="trailing"></slot>
  </div>
</column>
