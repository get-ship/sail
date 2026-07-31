/** postMessage protocol with studio-bridge (`channel: 'sail-studio'`). */

export const STUDIO_CHANNEL = 'sail-studio' as const
export const LS_FAB = 'sail-studio.fab'

export type BridgeRect = { left: number; top: number; width: number; height: number }

export type StudioIncoming =
  | { channel: typeof STUDIO_CHANNEL; type: 'ready' }
  | { channel: typeof STUDIO_CHANNEL; type: 'open'; open: boolean }
  | { channel: typeof STUDIO_CHANNEL; type: 'mode'; mode: string }
  | {
      channel: typeof STUDIO_CHANNEL
      type: 'select'
      nodeId?: string | null
      selector?: string
    }
  | { channel: typeof STUDIO_CHANNEL; type: 'annotate'; rect: BridgeRect }
  | {
      channel: typeof STUDIO_CHANNEL
      type: 'screenshot'
      kind?: string
      rect?: BridgeRect
    }
  | { channel: typeof STUDIO_CHANNEL; type: 'hotkey'; key: string }
  | { channel: typeof STUDIO_CHANNEL; type: 'settings'; fabVisible?: boolean }

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
    window.postMessage(
      { channel: STUDIO_CHANNEL, type: 'settings', fabVisible: visible },
      '*',
    )
  }
}

export function readFabVisible(): boolean {
  try {
    return localStorage.getItem(LS_FAB) !== '0'
  } catch {
    return true
  }
}
