<!--
  <sail-message> — one chat bubble. Content via default slot; actions via trailing.
  from / content-type / status ride data-state (disjoint vocabularies) for
  group-state paint. from is message kind, not an ARIA role.
-->

<scope>
  props: {
    from: 'assistant' | 'user' | 'system' | 'tool' = 'assistant'
    'content-type': 'text' | 'code' | 'drawing' | 'terminal-snapshot' | 'tool-call' | 'citation' | 'image' = 'text'
    timestamp = ''
    status: 'sent' | 'sending' | 'error' = 'sent'
  }

  style host = {
    box: maxW(full),
    paint: group().fg(foreground),
    text: fontFamily(body),
  }
  style meta = {
    box: padX(half),
    paint: fg(muted-foreground),
    text: fontSize(xs),
  }
  style fromLabel = { text: fontWeight(medium) }
  style bubble = {
    box: padY(2).padX(3),
    paint: radius(lg).bg(surface-1).borderWidth(1).border(border).fg(foreground),
    text: fontSize(base).leading(normal),
    when: {
      groupStateUser: { paint: bg(hover-bg).borderWidth(1).border(primary) },
      groupStateSystem: { paint: bg(muted).fg(muted-foreground) },
      groupStateTool: { paint: bg(hover-bg).borderWidth(1).border(accent) },
      groupStateError: { paint: borderWidth(1).border(destructive) },
      groupStateSending: { paint: opacity(75) },
      groupStateCode: { text: fontFamily(mono).fontSize(sm).whitespace(pre-wrap) },
      groupStateToolCall: { text: fontFamily(mono).fontSize(sm).whitespace(pre-wrap) },
      groupStateTerminalSnapshot: {
        paint: bg(surface-inverse).fg(background).borderWidth(1).border(border-subtle),
        text: fontFamily(mono).fontSize(sm).whitespace(pre-wrap),
      },
    },
  }
  // contents: no trailing box when empty so column gap never opens under it.
  style trailing = { box: display(contents) }
</scope>

<column
  gap="1"
  align={from() === 'user' ? 'end' : 'stretch'}
  data-state={from() + ' ' + contentType() + ' ' + status()}
  style(host)
>
  <row gap="2" align="baseline" style(meta)>
    <span style(fromLabel)>{from()}</span>
    <if {timestamp()}>
      <time datetime={timestamp()}>{timestamp()}</time>
    </if>
  </row>

  <div style(bubble)>
    <slot></slot>
  </div>

  <div style(trailing)>
    <slot name="trailing"></slot>
  </div>
</column>
