// The "server" for lesson 09, moved OUT of the controller into an imported
// client module. query() derives its cache key from a static member-path
// call on an imported binding (`inboxClient.messages.list`) — a bare local
// function reference gives the compiler nothing to key on.
//
// Note the repeated topics — a non-unique field is exactly the kind of
// thing an unreviewed key= reaches for. `list` returns a Promise, so
// query() routes it down the PULL path.

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
