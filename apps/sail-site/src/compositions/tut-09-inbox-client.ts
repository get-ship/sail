// Lesson 09 mock data client. query() keys on `inboxClient.messages.list`.
// Topics intentionally repeat so key={m.topic} is a planted review flaw.

export interface Message {
  id: number
  from: string
  subject: string
  topic: string
  tone: 'info' | 'warning' | 'destructive' | 'success'
  read: boolean
}

const MESSAGES: Message[] = [
  { id: 1, from: 'monitoring', subject: 'p95 back under 200ms', topic: 'latency', tone: 'success', read: true },
  { id: 2, from: 'jules', subject: 'refund for order #8841?', topic: 'billing', tone: 'warning', read: false },
  { id: 3, from: 'monitoring', subject: 'error budget 40% consumed', topic: 'latency', tone: 'info', read: false },
  { id: 4, from: 'sam', subject: 'card declined on renewal', topic: 'billing', tone: 'destructive', read: false },
  { id: 5, from: 'dana', subject: 'love the new tutorial', topic: 'feedback', tone: 'info', read: true },
]

export const inboxClient = {
  messages: {
    list: (): Promise<Message[]> =>
      new Promise((resolve) => setTimeout(() => resolve(MESSAGES), 700)),
  },
}
