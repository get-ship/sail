<controller>
import { controller, signal } from '@ship/sail-runtime'

interface MessageItem {
  id: string
  content: string
  from?: string
  contentType?: string
  timestamp?: string
  status?: string
}

const THREAD_ITEMS: MessageItem[] = [
  {
    id: 'msg-1',
    from: 'user',
    content: 'Can you help me understand the chat-elements pack?',
    timestamp: '2026-06-05T14:00:00Z',
    status: 'sent',
  },
  {
    id: 'msg-2',
    from: 'assistant',
    content:
      'The chat-elements pack provides six unbound primitives: sail-message, ' +
      'sail-message-list, sail-composer, sail-thinking-dots, sail-tool-call, and sail-citation.',
    timestamp: '2026-06-05T14:00:05Z',
    status: 'sent',
  },
  {
    id: 'msg-3',
    from: 'user',
    content: 'What does "unbound" mean?',
    timestamp: '2026-06-05T14:00:30Z',
    status: 'sent',
  },
  {
    id: 'msg-4',
    from: 'assistant',
    content:
      'Unbound means the primitives carry no Ship SDK dependency — no data ' +
      'fetching, no SSE subscriptions. Consumers supply their own arrays and ' +
      'mutate them via signals. The ship-agent-thread component wraps them with ' +
      'the Ship runtime binding.',
    timestamp: '2026-06-05T14:00:38Z',
    status: 'sent',
  },
  {
    id: 'msg-5',
    from: 'system',
    content: 'Tool call: search_code { query: "sail-message" }',
    contentType: 'tool-call',
    status: 'sent',
  },
]

export default controller(() => ({
    emptyItems: signal<MessageItem[]>([]),
    threadItems: signal<MessageItem[]>(THREAD_ITEMS),
  }))
</controller>

<scope>
  style page = { pad(6).size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style title = { margin({ b: 2 }).type({ size: 3xl, weight: bold }) }
  style lede = { fg(muted-foreground).type({ size: md }) }
  style sectionH = { margin({ b: 1 }).fg(primary).type({ size: lg, weight: semibold, family: mono }) }
  style sectionBody = { margin({ b: 4 }).fg(muted-foreground).type({ size: base }) }
  style card = { pad(4).bg(surface).border({ width: 1, color: border }).radius(md) }
  style cardLabel = { fg(muted-foreground).type({ size: xs, weight: medium, case: uppercase, tracking: wide }) }
  style listTall = { size({ h: 64 }).border({ width: 1, color: border }).radius(md) }
  style listShort = { size({ h: 48 }).border({ width: 1, color: border }).radius(md) }
  style mutedBase = { fg(muted-foreground).type({ size: base }) }
  style mutedXs = { fg(muted-foreground).type({ size: xs }) }
  style chip = { pad({ x: 2, y: 1 }).type({ size: xs }) }
</scope>

<column gap="8" style(page)>
  <header>
    <h1 style(title)>sail/chat-elements</h1>
    <p style(lede)>
      Unbound multi-modal chat primitives — message · message-list · composer ·
      thinking-dots · tool-call · citation. Pure framework tier; no SDK reads.
    </p>
  </header>
  <section>
    <h2 style(sectionH)>sail-citation</h2>
    <p style(sectionBody)>
      Numbered source pill. Clicking emits <code>sail-citation-open</code>.
    </p>

    <column gap="3">
      <column gap="3" style(card)>
        <h3 style(cardLabel)>index 1 — short label</h3>
        <sail-citation source="Ship docs" index={1}></sail-citation>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>index 2 — URL source</h3>
        <sail-citation source="https://ship.example.com/reference" index={2}></sail-citation>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>index 3 — long label (truncates)</h3>
        <sail-citation source="A very long citation label that will be clipped by the max-width" index={3}></sail-citation>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>inline inside prose</h3>
        <p>
          The agent used three sources
          <sail-citation source="Ship docs" index={1}></sail-citation>
          <sail-citation source="RFC 9110" index={2}></sail-citation>
          to answer your question.
        </p>
      </column>
    </column>
  </section>
  <section>
    <h2 style(sectionH)>sail-composer</h2>
    <p style(sectionBody)>
      Message input + send control. Multiline grow is the product default
      (textarea + autosize); set multiline false for a single-line field.
      Emits <code>sail-composer-submit</code> on send. Enter submits;
      Shift+Enter inserts a newline when multiline.
    </p>

    <column gap="3">
      <column gap="3" style(card)>
        <h3 style(cardLabel)>default — multiline grow, placeholder</h3>
        <sail-composer placeholder="Message the agent…"></sail-composer>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>single-line — multiline false</h3>
        <sail-composer placeholder="Quick reply…" multiline={false}></sail-composer>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>disabled</h3>
        <sail-composer placeholder="Sending…" disabled></sail-composer>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>with trailing slot — extra action beside Send</h3>
        <sail-composer placeholder="Ask anything…">
          <button slot="trailing" type="button" style(chip)>Mic</button>
        </sail-composer>
      </column>
    </column>
  </section>
  <section>
    <h2 style(sectionH)>sail-message</h2>
    <p style(sectionBody)>
      Single chat bubble. Role drives alignment + tone; content-type drives inner
      shape; status tints the border. Trailing slot for per-message actions.
    </p>

    <column gap="3">
      <column gap="3" style(card)>
        <h3 style(cardLabel)>from: assistant — text (default)</h3>
        <sail-message from="assistant" timestamp="2026-06-05T14:00:00Z">
          Here is the information you requested about Sail components.
        </sail-message>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>from: user</h3>
        <sail-message from="user" timestamp="2026-06-05T14:01:00Z">
          Can you show me the chat-elements pack?
        </sail-message>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>from: system</h3>
        <sail-message from="system">
          You are a helpful assistant with access to Ship tools.
        </sail-message>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>from: tool</h3>
        <sail-message from="tool" content-type="text">
          Tool output: 42 results found.
        </sail-message>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>content-type: code</h3>
        <sail-message from="assistant" content-type="code">
          const x = signal(0);
const doubled = derived(() => x() * 2);
        </sail-message>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>content-type: terminal-snapshot</h3>
        <sail-message from="assistant" content-type="terminal-snapshot">
          $ cargo run -p sail-compiler -- build page.sail
compiled 1 file(s)
        </sail-message>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>status: sending</h3>
        <sail-message from="user" status="sending">
          Sending this message now…
        </sail-message>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>status: error</h3>
        <sail-message from="user" status="error">
          This message failed to send.
        </sail-message>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>with trailing slot — copy action</h3>
        <sail-message from="assistant" timestamp="2026-06-05T14:02:00Z">
          The answer is forty-two.
          <span slot="trailing" style(mutedXs)>Copy</span>
        </sail-message>
      </column>
    </column>
  </section>
  <section>
    <h2 style(sectionH)>sail-message-list</h2>
    <p style(sectionBody)>
      Keyed, windowed, auto-scrolling column. Items via <code>items</code>;
      <code>sticky-bottom</code> pins to newest. Empty state via the
      <code>empty</code> slot.
    </p>

    <column gap="3">
      <column gap="3" style(card)>
        <h3 style(cardLabel)>empty state — no items</h3>
        <column style(listShort)>
          <sail-message-list items={emptyItems()}>
            <span slot="empty" style(mutedBase)>
              No messages yet. Start the conversation.
            </span>
          </sail-message-list>
        </column>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>sticky-bottom=true — pre-loaded thread</h3>
        <column style(listTall)>
          <sail-message-list items={threadItems()}>
            <span slot="empty">No messages.</span>
          </sail-message-list>
        </column>
      </column>
    </column>
  </section>
  <section>
    <h2 style(sectionH)>sail-thinking-dots</h2>
    <p style(sectionBody)>
      Three bouncing dots signalling an assistant composing a reply. Pure
      presentational; no controller. Screen-reader-friendly live region.
    </p>

    <column gap="3">
      <column gap="3" style(card)>
        <h3 style(cardLabel)>size: sm | md (default) | lg</h3>
        <row gap="4">
          <sail-thinking-dots size="sm"></sail-thinking-dots>
          <sail-thinking-dots size="md"></sail-thinking-dots>
          <sail-thinking-dots size="lg"></sail-thinking-dots>
        </row>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>tone: default | primary | info | success | warning | destructive | accent | muted</h3>
        <row gap="4">
          <sail-thinking-dots></sail-thinking-dots>
          <sail-thinking-dots tone="primary"></sail-thinking-dots>
          <sail-thinking-dots tone="info"></sail-thinking-dots>
          <sail-thinking-dots tone="success"></sail-thinking-dots>
          <sail-thinking-dots tone="warning"></sail-thinking-dots>
          <sail-thinking-dots tone="destructive"></sail-thinking-dots>
          <sail-thinking-dots tone="accent"></sail-thinking-dots>
          <sail-thinking-dots tone="muted"></sail-thinking-dots>
        </row>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>custom label — overrides announced text</h3>
        <sail-thinking-dots label="Model is composing" size="md" tone="primary"></sail-thinking-dots>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>inline inside a message bubble</h3>
        <sail-message from="assistant">
          <sail-thinking-dots size="sm" tone="muted"></sail-thinking-dots>
        </sail-message>
      </column>
    </column>
  </section>
  <section>
    <h2 style(sectionH)>sail-tool-call</h2>
    <p style(sectionBody)>
      Collapsible record of one tool invocation. Header shows name + status;
      body shows args (default slot) and result (<code>result</code> slot).
    </p>

    <column gap="3">
      <column gap="3" style(card)>
        <h3 style(cardLabel)>status: running — body collapsed</h3>
        <sail-tool-call name="list_files" status="running">
          path: /workspace/src
        </sail-tool-call>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>status: ok — body collapsed</h3>
        <sail-tool-call name="read_file" status="ok">
          path: /workspace/src/lib.rs
          <span slot="result">pub fn hello() -> String</span>
        </sail-tool-call>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>status: error — body collapsed</h3>
        <sail-tool-call name="run_tests" status="error">
          suite: unit
          <span slot="result">Error: test runner not found</span>
        </sail-tool-call>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>open=true — body expanded on load</h3>
        <sail-tool-call name="search_code" status="ok" open>
          query: sail-message, scope: packages/
          <span slot="result">Found 7 matches in 3 files.</span>
        </sail-tool-call>
      </column>
    </column>
  </section>
  </column>
