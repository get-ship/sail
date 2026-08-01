/**
 * Assign chat-elements host properties (same pattern as assignTreeHost / assignTableHost).
 * sail-message-list: `items`; sail-composer: `attachments`.
 */

export type ChatMessageItem = {
  id: string
  content?: string
  role?: string
  contentType?: string
  timestamp?: string
  status?: string
}

export function assignMessageListHost(sel: string, items: ChatMessageItem[]): void {
  if (typeof document === 'undefined') return
  const el = document.querySelector(sel) as (HTMLElement & { items?: ChatMessageItem[] }) | null
  if (!el) return
  // Always replace so the list reconciles after reply / load.
  el.items = items
}

export function assignComposerAttachments(sel: string, attachments: unknown[]): void {
  if (typeof document === 'undefined') return
  const el = document.querySelector(sel) as (HTMLElement & { attachments?: unknown[] }) | null
  if (!el) return
  el.attachments = attachments
}
