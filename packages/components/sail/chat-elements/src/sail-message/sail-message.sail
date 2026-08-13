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
    box: size({ max-w: full }),
    paint: group().ink({ color: foreground }),
    text: type({ family: body }),
  }
  style meta = {
    box: space({ pad-x: half }),
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs }),
  }
  style fromLabel = { text: type({ weight: medium }) }
  style bubble = {
    box: space({ pad-x: 3, pad-y: 2 }),
    paint: radius(lg).fill({ color: surface-1 }).border({ color: border, width: 1 }).ink({ color: foreground }),
    text: type({ size: base, leading: normal }),
    when: {
      groupStateUser: { paint: fill({ color: hover-bg }).border({ color: primary, width: 1 }) },
      groupStateSystem: { paint: fill({ color: muted }).ink({ color: muted-foreground }) },
      groupStateTool: { paint: fill({ color: hover-bg }).border({ color: accent, width: 1 }) },
      groupStateError: { paint: border({ color: destructive, width: 1 }) },
      groupStateSending: { paint: filter({ opacity: 75 }) },
      groupStateCode: { text: type({ family: mono, size: sm, wrap: pre-wrap }) },
      groupStateToolCall: { text: type({ family: mono, size: sm, wrap: pre-wrap }) },
      groupStateTerminalSnapshot: {
        paint: fill({ color: surface-inverse }).ink({ color: background }).border({ color: border-subtle, width: 1 }),
        text: type({ family: mono, size: sm, wrap: pre-wrap }),
      },
    },
  }
  // contents: no trailing box when empty so column gap never opens under it.
  style trailing = { box: flow({ display: contents }) }
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
