/** postMessage protocol with studio-bridge (`channel: 'sail-studio'`). */

export const STUDIO_CHANNEL = 'sail-studio' as const
export const LS_FAB = 'sail-studio.fab'

export type BridgeRect = { left: number; top: number; width: number; height: number }

/**
 * Discriminated studio channel ops (typed payloads).
 * Bodies for {@link postStudioMessage} — `channel` is applied by the poster.
 * Only ops that real senders/receivers use (bridge, shell, studio app).
 */
export type StudioOp =
  | {
      type: 'select'
      nodeId?: string | null
      selector?: string
      path?: string
      line?: number
      col?: number
    }
  | { type: 'annotate'; rect: BridgeRect }
  | { type: 'screenshot'; kind?: string; rect?: BridgeRect }
  | { type: 'settings'; fabVisible?: boolean; open?: boolean }
  | { type: 'close' }
  | {
      type: 'theme-preview'
      themeId: string
      density: string
      patch: Record<string, string>
    }

/** Full wire message on the studio channel. */
export type StudioMessage = { channel: typeof STUDIO_CHANNEL } & StudioOp

/** Incoming wire messages (same union as posted ops + channel). */
export type StudioIncoming = StudioMessage

export function isStudioMessage(data: unknown): data is StudioIncoming {
  return (
    !!data &&
    typeof data === 'object' &&
    (data as { channel?: string }).channel === STUDIO_CHANNEL
  )
}

/** Persist FAB preference + tell the bridge (and any other listeners). */
export function publishFabVisible(visible: boolean): void {
  try {
    localStorage.setItem(LS_FAB, visible ? '1' : '0')
  } catch {
    /* */
  }
  if (typeof window !== 'undefined') {
    const msg: StudioMessage = {
      channel: STUDIO_CHANNEL,
      type: 'settings',
      fabVisible: visible,
    }
    window.postMessage(msg, '*')
  }
}

export function readFabVisible(): boolean {
  try {
    return localStorage.getItem(LS_FAB) !== '0'
  } catch {
    return true
  }
}
