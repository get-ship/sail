/** Preview fetch single-flight + region paint glue for edit-surface VM. */
import { getShellState } from '@ship/sail-studio-shell'
import { activeDoc, getEditState, refreshPreview } from '@ship/studio/edit'
import {
  paintRegionOverlay,
  type RegionBoxUi,
} from '../edit/region-overlay'

export type PreviewFlight = {
  inflight: string | null
  failedPath: string | null
}

export function buildRegionBoxes(): RegionBoxUi[] {
  const es = getEditState()
  const doc = activeDoc()
  const shell = getShellState()
  const vw = shell.viewport || 1440
  const vh = Math.round((vw * 9) / 16)
  const selected = es.selectionNodeId
  const hover = es.hoverTip.visible ? es.hoverTip.nodeId : null
  return (doc?.preview.regions ?? [])
    .filter((r) => r.w > 0 && r.h > 0)
    .map((r) => ({
      id: r.nodeId,
      left: `${((r.x / vw) * 100).toFixed(2)}%`,
      top: `${((r.y / vh) * 100).toFixed(2)}%`,
      width: `${((r.w / vw) * 100).toFixed(2)}%`,
      height: `${((r.h / vh) * 100).toFixed(2)}%`,
      selected: r.nodeId === selected || r.nodeId === hover,
      key: r.nodeId,
    }))
}

export function paintRegions(): RegionBoxUi[] {
  const boxes = buildRegionBoxes()
  queueMicrotask(() => paintRegionOverlay(boxes))
  return boxes
}

/** Single-flight preview load; fails soft and will not storm. */
export function ensurePreviewOnce(
  flight: PreviewFlight,
  onDone: () => void,
): void {
  const es = getEditState()
  const doc = activeDoc()
  if (!es.activePath || !doc) return
  if (es.view !== 'preview' && es.view !== 'layout') return
  if (doc.preview.imagePng) {
    flight.failedPath = null
    return
  }
  if (flight.inflight === es.activePath) return
  if (flight.failedPath === es.activePath) return
  const path = es.activePath
  flight.inflight = path
  const shell = getShellState()
  const w = shell.viewport
  void refreshPreview({
    theme: shell.themeId,
    mode: shell.modeId,
    viewport: { width: w, height: Math.round((w * 9) / 16) },
  })
    .then(() => {
      flight.inflight = null
      flight.failedPath = activeDoc()?.preview.imagePng ? null : path
      onDone()
    })
    .catch(() => {
      flight.inflight = null
      flight.failedPath = path
      onDone()
    })
}
