/**
 * Assign chat-elements host properties (message list items / composer attachments).
 *
 * Ambient `document.querySelector` host mutation is banned
 * (`sail-no-ambient-dom`). Until templates bind prop:items / prop:attachments,
 * these are no-ops.
 */

export type ChatMessageItem = {
  id: string
  content?: string
  from?: string
  contentType?: string
  timestamp?: string
  status?: string
}

export function assignMessageListHost(_sel: string, _items: ChatMessageItem[]): void {
  /* no-op until prop:items bind */
}

export function assignComposerAttachments(_sel: string, _attachments: unknown[]): void {
  /* no-op until prop:attachments bind */
}
