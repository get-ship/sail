// sail-media engine — the ViewerRenderer contract + the sanctioned lifecycle-
// event helpers. A contract + helper, NOT a component and NOT a runtime util:
// it lives in this pack because it is this family's shared shape. Every renderer
// (the sail-media-* leaf renderers, plus the composed primitive targets the
// @ship/viewer router dispatches to) emits these events so the router can
// observe any renderer through ONE listener.
//
// Events go through the exported free `emit(host, …)` (host-api.ts:60) — the
// single sanctioned wrapper (it builds `new CustomEvent(name, { detail,
// bubbles: true, composed: true })`). NOT `host.emit!` (optional-typed), NOT an
// open-coded `dispatchEvent`. The param is `EventTarget` to match `emit`.
//
// Authority: build-spec §"Backbone #2 — the ViewerRenderer contract".

import { emit } from '@ship/sail-runtime/internal'
import type { ViewerSource } from './types'

export interface ViewerRendererProps {
  /** prop:src identity (live) — a resolved ViewerSource / Blob / ArrayBuffer /
   *  object-url / text the consumer already fetched. The renderer never
   *  fetches its own bytes. */
  src?: ViewerSource | Blob | ArrayBuffer | string
}

export interface ViewerRendererEvents {
  'sail-viewer-load': void
  'sail-viewer-error': { message: string }
  /** pdf page-streaming only. */
  'sail-viewer-progress'?: { loaded: number; total: number }
}

/** Fired on first successful render/parse. */
export function emitViewerLoad(host: EventTarget): void {
  emit(host, 'sail-viewer-load')
}

/** Fired on parse/decode failure with the computed message. */
export function emitViewerError(host: EventTarget, message: string): void {
  emit(host, 'sail-viewer-error', { message })
}

/** Fired for pdf page-stream progress. */
export function emitViewerProgress(host: EventTarget, loaded: number, total: number): void {
  emit(host, 'sail-viewer-progress', { loaded, total })
}
