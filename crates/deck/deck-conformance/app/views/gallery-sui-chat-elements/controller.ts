import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

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
      'The chat-elements pack provides six unbound primitives: sui-message, ' +
      'sui-message-list, sui-composer, sui-thinking-dots, sui-tool-call, and sui-citation.',
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
    content: 'Tool call: search_code { query: "sui-message" }',
    contentType: 'tool-call',
    status: 'sent',
  },
]

export default controller<Scope>({
  scope: () => ({
    emptyItems: signal<MessageItem[]>([]),
    threadItems: signal<MessageItem[]>(THREAD_ITEMS),
  }),
})
