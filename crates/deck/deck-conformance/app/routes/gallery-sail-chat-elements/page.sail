<controller>
import { controller, signal } from '@ship/sail-runtime'

interface MessageItem {
  id: string
  content: string
  role?: string
  contentType?: string
  timestamp?: string
  status?: string
}

const THREAD_ITEMS: MessageItem[] = [
  {
    id: 'msg-1',
    role: 'user',
    content: 'Can you help me understand the chat-elements pack?',
    timestamp: '2026-06-05T14:00:00Z',
    status: 'sent',
  },
  {
    id: 'msg-2',
    role: 'assistant',
    content:
      'The chat-elements pack provides six unbound primitives: sail-message, ' +
      'sail-message-list, sail-composer, sail-thinking-dots, sail-tool-call, and sail-citation.',
    timestamp: '2026-06-05T14:00:05Z',
    status: 'sent',
  },
  {
    id: 'msg-3',
    role: 'user',
    content: 'What does "unbound" mean?',
    timestamp: '2026-06-05T14:00:30Z',
    status: 'sent',
  },
  {
    id: 'msg-4',
    role: 'assistant',
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
    role: 'system',
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

<column gap="8" sx="max-w-breakpoint-tablet mx-auto p-6 font-body text-foreground">
  <header>
    <h1 sx="text-3xl font-bold mb-2">sui/chat-elements</h1>
    <p sx="text-md text-muted-foreground">
      Unbound multi-modal chat primitives — message · message-list · composer ·
      thinking-dots · tool-call · citation. Pure framework tier; no SDK reads.
    </p>
  </header>

  <!-- ── sail-citation ──────────────────────────────────────────── -->
  <section>
    <h2 sx="text-lg font-semibold font-mono text-primary mb-1">sail-citation</h2>
    <p sx="text-base text-muted-foreground mb-4">
      Numbered source pill. Clicking emits <code>sail-citation-open</code>.
    </p>

    <column gap="3">
      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">index 1 — short label</h3>
        <sail-citation source="Ship docs" index={1}></sail-citation>
      </column>

      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">index 2 — URL source</h3>
        <sail-citation source="https://ship.example.com/reference" index={2}></sail-citation>
      </column>

      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">index 3 — long label (truncates)</h3>
        <sail-citation source="A very long citation label that will be clipped by the max-width" index={3}></sail-citation>
      </column>

      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">inline inside prose</h3>
        <p>
          The agent used three sources
          <sail-citation source="Ship docs" index={1}></sail-citation>
          <sail-citation source="RFC 9110" index={2}></sail-citation>
          to answer your question.
        </p>
      </column>
    </column>
  </section>

  <!-- ── sail-composer ──────────────────────────────────────────── -->
  <section>
    <h2 sx="text-lg font-semibold font-mono text-primary mb-1">sail-composer</h2>
    <p sx="text-base text-muted-foreground mb-4">
      Message input + send control. Single-line by default; <code>multiline</code>
      grows to a textarea. Emits <code>sail-composer-submit</code> on send.
    </p>

    <column gap="3">
      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">default — single-line, placeholder</h3>
        <sail-composer placeholder="Message the agent…"></sail-composer>
      </column>

      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">multiline — Shift+Enter for newlines</h3>
        <sail-composer placeholder="Write a long message…" multiline></sail-composer>
      </column>

      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">disabled</h3>
        <sail-composer placeholder="Sending…" disabled></sail-composer>
      </column>

      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">with trailing slot — extra action beside Send</h3>
        <sail-composer placeholder="Ask anything…">
          <button slot="trailing" type="button" sx="text-xs px-2 py-1">Mic</button>
        </sail-composer>
      </column>
    </column>
  </section>

  <!-- ── sail-message ────────────────────────────────────────────── -->
  <section>
    <h2 sx="text-lg font-semibold font-mono text-primary mb-1">sail-message</h2>
    <p sx="text-base text-muted-foreground mb-4">
      Single chat bubble. Role drives alignment + tone; content-type drives inner
      shape; status tints the border. Trailing slot for per-message actions.
    </p>

    <column gap="3">
      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">role: assistant — text (default)</h3>
        <sail-message role="assistant" timestamp="2026-06-05T14:00:00Z">
          Here is the information you requested about SUI components.
        </sail-message>
      </column>

      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">role: user</h3>
        <sail-message role="user" timestamp="2026-06-05T14:01:00Z">
          Can you show me the chat-elements pack?
        </sail-message>
      </column>

      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">role: system</h3>
        <sail-message role="system">
          You are a helpful assistant with access to Ship tools.
        </sail-message>
      </column>

      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">role: tool</h3>
        <sail-message role="tool" content-type="text">
          Tool output: 42 results found.
        </sail-message>
      </column>

      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">content-type: code</h3>
        <sail-message role="assistant" content-type="code">
          const x = signal(0);
const doubled = derived(() => x() * 2);
        </sail-message>
      </column>

      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">content-type: terminal-snapshot</h3>
        <sail-message role="assistant" content-type="terminal-snapshot">
          $ cargo run -p sail-compiler -- build page.sui
compiled 1 file(s)
        </sail-message>
      </column>

      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">status: sending</h3>
        <sail-message role="user" status="sending">
          Sending this message now…
        </sail-message>
      </column>

      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">status: error</h3>
        <sail-message role="user" status="error">
          This message failed to send.
        </sail-message>
      </column>

      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">with trailing slot — copy action</h3>
        <sail-message role="assistant" timestamp="2026-06-05T14:02:00Z">
          The answer is forty-two.
          <span slot="trailing" sx="text-xs text-muted-foreground">Copy</span>
        </sail-message>
      </column>
    </column>
  </section>

  <!-- ── sail-message-list ───────────────────────────────────────── -->
  <section>
    <h2 sx="text-lg font-semibold font-mono text-primary mb-1">sail-message-list</h2>
    <p sx="text-base text-muted-foreground mb-4">
      Keyed, windowed, auto-scrolling column. Items via <code>items</code>;
      <code>sticky-bottom</code> pins to newest. Empty state via the
      <code>empty</code> slot.
    </p>

    <column gap="3">
      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">empty state — no items</h3>
        <column sx="h-48 border border-border rounded-md">
          <sail-message-list items={emptyItems()}>
            <span slot="empty" sx="text-muted-foreground text-base">
              No messages yet. Start the conversation.
            </span>
          </sail-message-list>
        </column>
      </column>

      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">sticky-bottom=true — pre-loaded thread</h3>
        <column sx="h-64 border border-border rounded-md">
          <sail-message-list items={threadItems()}>
            <span slot="empty">No messages.</span>
          </sail-message-list>
        </column>
      </column>
    </column>
  </section>

  <!-- ── sail-thinking-dots ──────────────────────────────────────── -->
  <section>
    <h2 sx="text-lg font-semibold font-mono text-primary mb-1">sail-thinking-dots</h2>
    <p sx="text-base text-muted-foreground mb-4">
      Three bouncing dots signalling an assistant composing a reply. Pure
      presentational; no controller. Screen-reader-friendly live region.
    </p>

    <column gap="3">
      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">size: sm | md (default) | lg</h3>
        <row gap="4">
          <sail-thinking-dots size="sm"></sail-thinking-dots>
          <sail-thinking-dots size="md"></sail-thinking-dots>
          <sail-thinking-dots size="lg"></sail-thinking-dots>
        </row>
      </column>

      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">tone: default | primary | info | success | warning | destructive | accent | muted</h3>
        <row gap="4">
          <sail-thinking-dots tone="default"></sail-thinking-dots>
          <sail-thinking-dots tone="primary"></sail-thinking-dots>
          <sail-thinking-dots tone="info"></sail-thinking-dots>
          <sail-thinking-dots tone="success"></sail-thinking-dots>
          <sail-thinking-dots tone="warning"></sail-thinking-dots>
          <sail-thinking-dots tone="destructive"></sail-thinking-dots>
          <sail-thinking-dots tone="accent"></sail-thinking-dots>
          <sail-thinking-dots tone="muted"></sail-thinking-dots>
        </row>
      </column>

      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">custom label — overrides announced text</h3>
        <sail-thinking-dots label="Model is composing" size="md" tone="primary"></sail-thinking-dots>
      </column>

      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">inline inside a message bubble</h3>
        <sail-message role="assistant">
          <sail-thinking-dots size="sm" tone="muted"></sail-thinking-dots>
        </sail-message>
      </column>
    </column>
  </section>

  <!-- ── sail-tool-call ──────────────────────────────────────────── -->
  <section>
    <h2 sx="text-lg font-semibold font-mono text-primary mb-1">sail-tool-call</h2>
    <p sx="text-base text-muted-foreground mb-4">
      Collapsible record of one tool invocation. Header shows name + status dot;
      body shows args (default slot) and result (<code>result</code> slot).
    </p>

    <column gap="3">
      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">status: running — body collapsed</h3>
        <sail-tool-call name="list_files" status="running">
          { "path": "/workspace/src" }
        </sail-tool-call>
      </column>

      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">status: ok — body collapsed</h3>
        <sail-tool-call name="read_file" status="ok">
          { "path": "/workspace/src/lib.rs" }
          <span slot="result">pub fn hello() -&gt; &amp;'static str { "world" }</span>
        </sail-tool-call>
      </column>

      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">status: error — body collapsed</h3>
        <sail-tool-call name="run_tests" status="error">
          { "suite": "unit" }
          <span slot="result">Error: test runner not found</span>
        </sail-tool-call>
      </column>

      <column gap="3" sx="bg-card border border-border rounded-md p-4">
        <h3 sx="text-xs font-medium uppercase tracking-wide text-muted-foreground">open=true — body expanded on load</h3>
        <sail-tool-call name="search_code" status="ok" open>
          { "query": "sail-message", "scope": "packages/" }
          <span slot="result">Found 7 matches in 3 files.</span>
        </sail-tool-call>
      </column>
    </column>
  </section>
</column>
