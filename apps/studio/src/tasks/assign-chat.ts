/**
 * Assign chat-elements host properties (message list items / composer attachments).
 *
 * Ambient `document.querySelector` host mutation is banned
 * (`sail-no-ambient-dom`). Until templates bind prop:items / prop:attachments,
 * these are no-ops.
 *
 * // SAIL-LINT-CHAT: migrate sail-message-list / sail-composer hosts to
 * // prop:items={…} / prop:attachments={…} identity binding in tasks-surface;
 * // delete this assign helper once bound.
 */

export type ChatMessageItem = {
  id: string
  content?: string
  role?: string
  contentType?: string
  timestamp?: string
  status?: string
}

export function assignMessageListHost(_sel: string, _items: ChatMessageItem[]): void {
  /* intentional no-op — see SAIL-LINT-CHAT above */
}

export function assignComposerAttachments(_sel: string, _attachments: unknown[]): void {
  /* intentional no-op — see SAIL-LINT-CHAT above */
}
