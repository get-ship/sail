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

  style host = { size({ max-w: 100% }).group().fg(foreground).type({ family: body }) }
  style meta = { pad({ x: half }).fg(muted-foreground).type({ size: xs }) }
  style fromLabel = { type({ weight: medium }) }
  style bubble = { pad({ x: 3, y: 2 }).radius(lg).bg(surface-1).border({ color: border, width: 1 }).fg(foreground).type({ size: base, leading: normal }), when: { groupStateUser: bg(hover-bg).border({ color: primary, width: 1 }), groupStateSystem: bg(muted).fg(muted-foreground), groupStateTool: bg(hover-bg).border({ color: accent, width: 1 }), groupStateError: border({ color: destructive, width: 1 }), groupStateSending: filter({ opacity: 75 }), groupStateCode: type({ family: mono, size: sm, wrap: pre-wrap }), groupStateToolCall: type({ family: mono, size: sm, wrap: pre-wrap }), groupStateTerminalSnapshot: bg(surface-inverse).fg(background).border({ color: border-subtle, width: 1 }).type({ family: mono, size: sm, wrap: pre-wrap }) } }
  // contents: no trailing box when empty so column gap never opens under it.
  style trailing = { display(contents) }
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
